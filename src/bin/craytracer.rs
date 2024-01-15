use clap::Parser;
use core::time;
use craytracer::{
    color::Color,
    sampling::sample_2d,
    scene::Scene,
    scene_parser::{scene_parser::parse_scene, tokenizer::ParserError},
    trace::path_trace,
};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

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

fn create_preview_window(width: usize, height: usize) -> Window {
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

    window.limit_update_rate(Some(time::Duration::from_millis(12)));

    window
}

fn show_preview(
    window: &mut Window,
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
    tiles: &Vec<(usize, usize, usize, usize)>,
    pixels: Arc<Mutex<Vec<f32>>>,
    receiver: Receiver<usize>,
) -> Vec<u32> {
    let mut preview_buffer = vec![0u32; width * height];

    // Initialize buffer to draw a checkerboard pattern
    for (x1, y1, x2, y2) in tiles.iter() {
        let tile_color = if ((x1 / tile_width) + (y1 / tile_height)) % 2 == 0 {
            0x999999
        } else {
            0xaaaaaa
        };
        for y in *y1..*y2 {
            for x in *x1..*x2 {
                preview_buffer[x + y * width] = tile_color;
            }
        }
    }

    let mut tile_count = 0;
    while tile_count < tiles.len() {
        window
            .update_with_buffer(&preview_buffer, width, height)
            .unwrap();

        if window.is_key_released(Key::Escape) {
            // Exit early if escape is pressed
            break;
        }

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
                    preview_buffer[offset] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                }
            }
            tile_count += 1;
        }
    }

    preview_buffer
}

fn render_tile<R>(
    tile: (usize, usize, usize, usize),
    scene: &Scene,
    width: usize,
    height: usize,
    pixels: &Arc<Mutex<Vec<f32>>>,
) where
    R: SeedableRng + Rng + ?Sized,
{
    let mut rng = R::from_entropy();

    let (x1, y1, x2, y2) = tile;
    for y in y1..y2 {
        for x in x1..x2 {
            let mut color = Color::BLACK;
            for _ in 0..scene.num_samples {
                let (dx, dy) = sample_2d(&mut rng);
                // We assume that the screen goes from (0, 0) at the
                // top left to (width - 1, height - 1) at the bottom
                // right. This is converted to [0, 1] x [0, 1] film
                // co-ordinates, starting at bottom left.
                let film_x = (x as f64 + dx) / (width - 1) as f64;
                let film_y = 1.0 - (y as f64 + dy) / (height - 1) as f64;

                let ray = scene.camera.sample(film_x, film_y);
                color += path_trace(&mut rng, ray, &scene);
            }
            color /= scene.num_samples as f64;

            let mut pixels = pixels.lock().unwrap();
            let (r, g, b) = color.into();
            let offset = x + y * width;
            pixels[3 * offset] = r;
            pixels[3 * offset + 1] = g;
            pixels[3 * offset + 2] = b;
        }
    }
}

fn render<R>(
    scene: &Scene,
    preview: bool,
    start: Instant,
) -> (Vec<f32>, Option<Window>, Option<Vec<u32>>)
where
    R: SeedableRng + Rng + ?Sized,
{
    let num_threads = num_cpus::get();

    let width = scene.film_width;
    let height = scene.film_height;
    let tile_width = 64;
    let tile_height = 64;
    let tiles = &generate_tiles(height, width, tile_width, tile_height);

    let pixels = Arc::new(Mutex::new(vec![0f32; width * height * 3]));

    let tile_index = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();

    let mut preview_window: Option<Window> = None;
    let mut preview_buffer: Option<Vec<u32>> = None;

    thread::scope(|scope| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            let tile_index = Arc::clone(&tile_index);
            let pixels = Arc::clone(&pixels);
            let sender = sender.clone();

            handles.push(scope.spawn(move || loop {
                let tile_index = tile_index.fetch_add(1, Ordering::SeqCst);
                if tile_index >= tiles.len() {
                    break;
                }

                render_tile::<R>(tiles[tile_index], scene, width, height, &pixels);

                if sender.send(tile_index).is_err() {
                    // The receiver has early exited
                    break;
                }

                let elapsed = start.elapsed().as_secs_f32();
                let estimate = elapsed * tiles.len() as f32 / (tile_index + 1) as f32;
                eprint!(
                    "\r{:3} / {:3} tiles {:5.1}s / {:5.1}s",
                    tile_index + 1,
                    tiles.len(),
                    elapsed,
                    estimate
                );
            }));
        }

        if preview {
            let mut window = create_preview_window(scene.film_width, scene.film_height);
            preview_buffer = Some(show_preview(
                &mut window,
                width,
                height,
                tile_width,
                tile_height,
                tiles,
                Arc::clone(&pixels),
                receiver,
            ));
            preview_window = Some(window);
        } else {
            for handle in handles {
                handle.join().unwrap();
            }
            println!();
        }
    });

    let pixels = pixels.lock().unwrap().clone();
    (pixels, preview_window, preview_buffer)
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    scene: String,

    #[clap(short, long, default_value_t = String::from("out.exr"))]
    output: String,

    #[clap(short, long)]
    preview: bool,
}

fn main() -> Result<(), ParserError> {
    let args = Cli::parse();

    let start = Instant::now();
    let input = std::fs::read_to_string(&args.scene).expect("Error reading scene file");
    let scene = match parse_scene(&input) {
        Ok(scene) => scene,
        Err(e) => {
            match e.location {
                Some(location) => println!("Error: {} at {}:{}", e.message, args.scene, location),
                None => println!("Error: {} in {}", e.message, args.scene),
            }
            return Ok(());
        }
    };
    println!("Scene constructed in {:?}", start.elapsed());

    // Render to a buffer
    let (pixels, preview_window, preview_buffer) = render::<SmallRng>(&scene, args.preview, start);
    println!("Rendering finished in {:?}", start.elapsed());

    // Save to file
    let image_buffer =
        image::Rgb32FImage::from_raw(scene.film_width as u32, scene.film_height as u32, pixels)
            .unwrap();
    image_buffer.save(&args.output).expect("Error saving file");
    println!("Output written to {}", &args.output);

    // Wait for user to close the preview window
    if let Some(mut preview_window) = preview_window {
        let mut preview_buffer = preview_buffer.unwrap();
        while preview_window.is_open() && !preview_window.is_key_released(Key::Escape) {
            preview_window
                .update_with_buffer(&mut preview_buffer, scene.film_width, scene.film_height)
                .unwrap();
        }
    }

    Ok(())
}
