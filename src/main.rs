use clap::Parser;
use color::Color;
use crossbeam::thread;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use sampling::sample_hemisphere;
use scene::Scene;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};
use trace::trace;
use vector::Vector;

use crate::{
    parser::parse_scene,
    scenes::{dragon, simple},
};

mod bounds;
mod bsdf;
mod bvh;
mod bxdf;
mod camera;
mod color;
mod constants;
mod intersection;
mod material;
mod obj;
mod parser;
mod pdf;
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
        while tile_count < tiles.len() {
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
                        // Gamma correction
                        .powf(1.0 / 2.2)
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
    #[clap(short, long, default_value_t = String::from("in.cry"))]
    scene: String,

    #[clap(short, long, default_value_t = String::from("out.exr"))]
    output: String,
}

// For verifying sampling methods
#[allow(unused)]
fn test_sampling() {
    let width = 400;
    let height = 400;
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

    let mut buffer = vec![0u32; width * height];
    let normal = Vector(0.0, 0.0, 1.0).normalized();

    while !window.is_key_released(Key::Escape) {
        for _ in 0..10 {
            let v = sample_hemisphere(&normal);
            let x = v.0;
            let y = v.1;
            let bx = ((x + 1.0) * 0.5 * (width - 1) as f64).round();
            let by = ((y + 1.0) * 0.5 * (height - 1) as f64).round();
            assert!(bx >= 0.0 && bx < width as f64);
            assert!(by >= 0.0 && by < height as f64);
            buffer[(bx + by * width as f64) as usize] = 0xFFFF0000;
        }
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

fn main() {
    // test_sampling();
    // return;

    let args = Cli::parse();

    let start = Instant::now();
    // let input = std::fs::read_to_string(&args.scene).expect("Error reading scene file");
    // let scene = parse_scene(&input).expect("Error parsing scene file");
    let scene = simple(250, 1);
    println!("Scene constructed in {:?}", start.elapsed());

    let width = scene.film_width as u32;
    let height = scene.film_height as u32;
    let pixels = render(&scene);
    println!("Rendering finished in {:?}", start.elapsed());

    let image_buffer = image::Rgb32FImage::from_raw(width, height, pixels).unwrap();
    image_buffer.save(&args.output).expect("Error saving file");

    println!("Output written to {}", &args.output);
}
