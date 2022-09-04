use clap::{Parser, ValueEnum};
use color::Color;
use crossbeam::thread;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use scene::Scene;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};
use trace::trace;

mod bounds;
mod bvh;
mod camera;
mod color;
mod constants;
mod intersection;
mod material;
mod obj;
mod primitive;
mod ray;
mod sampling;
mod scene;
mod scenes;
mod shape;
mod trace;
mod vector;

fn generate_tiles(
    height: usize,
    width: usize,
    tile_width: usize,
    tile_height: usize,
) -> Vec<(usize, usize, usize, usize)> {
    let mut tiles = Vec::new();
    for ty in (0..height).step_by(tile_height as usize) {
        for tx in (0..width).step_by(tile_width as usize) {
            tiles.push((
                tx,
                ty,
                (tx + tile_width).min(width),
                (ty + tile_height).min(height),
            ));
        }
    }
    tiles
}

fn setup_preview_window(
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
    tiles: &Vec<(usize, usize, usize, usize)>,
) -> (Vec<u32>, Window) {
    // The window library expects a vec<u32> buffer
    let mut buffer = vec![0u32; width * height];

    // Initialize buffer to draw a checkerboard pattern
    for (x1, y1, x2, y2) in tiles.iter() {
        let tile_color = if ((x1 / tile_width) + (y1 / tile_height)) % 2 == 0 {
            0x999999
        } else {
            0xaaaaaa
        };
        for y in *y1..*y2 {
            for x in *x1..*x2 {
                buffer[x + y * width] = tile_color;
            }
        }
    }

    let mut window = Window::new(
        "craytracer",
        width,
        height,
        WindowOptions {
            borderless: false,
            transparency: false,
            title: true,
            resize: true,
            scale: Scale::X1,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: false,
            none: false,
        },
    )
    .unwrap();
    window.update_with_buffer(&buffer, width, height).unwrap();

    (buffer, window)
}

fn render(scene: &Scene) -> Vec<f32> {
    let width = scene.film_width;
    let height = scene.film_height;
    let tile_width = 64;
    let tile_height = 64;
    let tiles = generate_tiles(height, width, tile_width, tile_height);

    let tiles = Arc::new(tiles);
    let pixels = Arc::new(Mutex::new(vec![0f32; width * height * 3]));
    let scene = Arc::new(scene);

    let tile_index = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();

    let num_threads = num_cpus::get();

    thread::scope(|scope| {
        for _ in 0..num_threads {
            let tile_index = Arc::clone(&tile_index);
            let tiles = Arc::clone(&tiles);
            let pixels = Arc::clone(&pixels);
            let scene = Arc::clone(&scene);
            let sender = sender.clone();

            scope.spawn(move |_| loop {
                let tile_index = tile_index.fetch_add(1, Ordering::SeqCst);
                if tile_index >= tiles.len() {
                    break;
                }

                let (x1, y1, x2, y2) = tiles[tile_index];
                for y in y1..y2 {
                    for x in x1..x2 {
                        let offset = x + y * width;
                        let color = scene.camera.sample(x, y, &scene);
                        let mut pixels = pixels.lock().unwrap();
                        let (r, g, b) = color.into();
                        pixels[3 * offset] = r;
                        pixels[3 * offset + 1] = g;
                        pixels[3 * offset + 2] = b;
                    }
                }

                sender
                    .send(tile_index)
                    .expect("Error sending to main thread");
            });
        }

        let (mut buffer, mut window) =
            setup_preview_window(width, height, tile_width, tile_height, &tiles);
        let mut tile_count = 0;
        while !window.is_key_released(Key::Escape) && tile_count < tiles.len() {
            if let Ok(tile_index) = receiver.try_recv() {
                let (x1, y1, x2, y2) = tiles[tile_index];
                for x in x1..x2 {
                    for y in y1..y2 {
                        let offset = x + y * width;
                        let pixels = pixels.lock().unwrap();
                        let (r, g, b) = Color {
                            r: pixels[3 * offset] as f64,
                            g: pixels[3 * offset + 1] as f64,
                            b: pixels[3 * offset + 2] as f64,
                        }
                        .gamma_correct(2.2)
                        .to_rgb();
                        buffer[offset] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    }
                }
                tile_count += 1;
                window.update_with_buffer(&buffer, width, height).unwrap();
            }
            window.update();
            std::thread::sleep(Duration::from_millis(16));
        }
    })
    .unwrap();

    let pixels = pixels.lock().unwrap().clone();
    pixels
}

#[derive(Parser)]
struct Cli {
    #[clap(arg_enum, value_parser)]
    scene: SceneName,

    #[clap(short, long, default_value_t = 64)]
    samples: usize,

    #[clap(short = 'S', long, default_value_t = 1)]
    scale: usize,

    #[clap(short, long, default_value_t = String::from("out.exr"))]
    output: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SceneName {
    Simple,
    RandomSpheres,
    Obj,
}

fn main() {
    let args = Cli::parse();

    let start = Instant::now();
    let scene = match args.scene {
        SceneName::Simple => scenes::simple(args.samples, args.scale),
        SceneName::RandomSpheres => scenes::random_spheres(args.samples, args.scale),
        SceneName::Obj => scenes::obj(args.samples, args.scale),
    };
    println!("Scene constructed in {:?}", start.elapsed());

    let width = scene.film_width as u32;
    let height = scene.film_height as u32;
    let pixels = render(&scene);
    println!("Rendering finished in {:?}", start.elapsed());

    let image_buffer = image::Rgb32FImage::from_raw(width, height, pixels).unwrap();
    image_buffer.save(&args.output).expect("Error saving file");

    println!("Output written to {}", &args.output);
}
