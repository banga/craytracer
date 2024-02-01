use clap::Parser;
use core::time;
use craytracer::{
    color::Color,
    path_integrator,
    sampling::samplers::{Sampler, SobolSampler},
    scene::Scene,
    scene_parser::{scene_parser::parse_scene, tokenizer::ParserError},
};
use log::{debug, error, info, LevelFilter};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use std::{
    ops::Range,
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
    num_samples: usize,
    tile_width: usize,
    tile_height: usize,
    sample_batch_size: usize,
) -> Vec<(Range<usize>, Range<usize>, Range<usize>)> {
    let mut tiles = Vec::new();
    for si in (0..num_samples).step_by(sample_batch_size) {
        for ty in (0..height).step_by(tile_height) {
            for tx in (0..width).step_by(tile_width) {
                tiles.push((
                    tx..(tx + tile_width).min(width),
                    ty..(ty + tile_height).min(height),
                    si..(si + sample_batch_size).min(num_samples),
                ));
            }
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

fn show_preview<C, F>(
    window: &mut Window,
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
    tiles: &Vec<(Range<usize>, Range<usize>, Range<usize>)>,
    pixels: Arc<Mutex<Vec<f32>>>,
    receiver: Receiver<usize>,
    mut on_click: C,
    on_finish: F,
) where
    C: FnMut(usize, usize),
    F: FnOnce(),
{
    let mut preview_buffer = vec![0u32; width * height];

    // Initialize buffer to draw a checkerboard pattern
    for (xr, yr, sr) in tiles.iter() {
        if sr.start > 0 {
            continue;
        }
        let tile_color = if ((xr.start / tile_width) + (yr.start / tile_height)) % 2 == 0 {
            0x999999
        } else {
            0xaaaaaa
        };
        for y in yr.clone() {
            for x in xr.clone() {
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
            let (xr, yr, sr) = &tiles[tile_index];
            for x in xr.clone() {
                for y in yr.clone() {
                    let offset = x + y * width;
                    let pixels = pixels.lock().unwrap();
                    let (r, g, b) = ((Color {
                        r: pixels[3 * offset] as f64,
                        g: pixels[3 * offset + 1] as f64,
                        b: pixels[3 * offset + 2] as f64,
                    }) / (sr.end as f64))
                        .to_rgb();
                    preview_buffer[offset] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                }
            }
            tile_count += 1;
        }
    }

    on_finish();

    // Wait for user to close the preview window. If the left mouse button is
    // released, invoke on_click with the pixel value.
    let mut is_left_button_down = false;
    while window.is_open() && !window.is_key_released(Key::Escape) {
        window
            .update_with_buffer(&mut preview_buffer, width, height)
            .unwrap();

        // TODO: Sometimes the mouse events stop reporting if you click too
        // often.
        if window.get_mouse_down(minifb::MouseButton::Left) {
            is_left_button_down = true;
        } else if is_left_button_down {
            if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Pass) {
                on_click(x as usize, y as usize)
            }
            is_left_button_down = false;
        } else {
            is_left_button_down = false;
        }
    }
}

#[allow(non_snake_case)]
#[inline]
fn render_pixel<S>(sampler: &mut S, x: usize, y: usize, sample_index: usize, scene: &Scene) -> Color
where
    S: Sampler,
{
    sampler.start_pixel(x, y, sample_index);
    let film_sample = sampler.sample_2d();
    let lens_sample = sampler.sample_2d();

    let ray = scene.camera.sample((film_sample, lens_sample), x, y);

    // TODO: Allow picking integrator from command line
    let L = path_integrator::estimate_Li(sampler, ray, &scene);
    // let L = simple_integrator::estimate_Li(sampler, ray, &scene);
    L
}

fn render_tile<S>(
    sampler: &mut S,
    tile: &(Range<usize>, Range<usize>, Range<usize>),
    scene: &Scene,
    width: usize,
    pixels: &Arc<Mutex<Vec<f32>>>,
) where
    S: Sampler,
{
    let (x_range, y_range, sample_range) = tile;
    for y in y_range.clone() {
        for x in x_range.clone() {
            let mut color = Color::BLACK;
            for sample_index in sample_range.clone() {
                color += render_pixel(sampler, x, y, sample_index, scene);
            }

            let (r, g, b) = color.into();
            let offset = x + y * width;

            let mut pixels = pixels.lock().unwrap();
            pixels[3 * offset] += r;
            pixels[3 * offset + 1] += g;
            pixels[3 * offset + 2] += b;
        }
    }
}

fn update_render_progress(start: Instant, num_rendered: usize, num_total: usize) {
    let elapsed = start.elapsed().as_secs_f32();
    let estimate = elapsed * num_total as f32 / num_rendered as f32;
    eprint!(
        "\r{:3} / {:3} tiles {:5.1}s / {:5.1}s ({:5.1}s remaining)",
        num_rendered,
        num_total,
        elapsed,
        estimate,
        estimate - elapsed
    );
}

fn render<S, F>(scene: &Scene, mut sampler: S, preview: bool, start: Instant, on_render_finish: F)
where
    S: Sampler + Send,
    F: FnOnce(Vec<f32>),
{
    let num_threads = num_cpus::get();

    let (width, height) = scene.film_bounds();
    let tile_width = 64;
    let tile_height = 64;
    let sample_batch_size = 8;
    let num_samples = sampler.num_samples();
    let tiles = &generate_tiles(
        height,
        width,
        num_samples,
        tile_width,
        tile_height,
        sample_batch_size,
    );

    let pixels = Arc::new(Mutex::new(vec![0f32; width * height * 3]));
    let on_finish = || {
        let mut pixels = pixels.lock().unwrap().clone();
        for pixel in pixels.iter_mut() {
            *pixel /= num_samples as f32;
        }
        on_render_finish(pixels);
    };

    let tile_index = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();

    debug!(
        "Rendering {} pixels in {} tiles using {} threads",
        pixels.lock().unwrap().len(),
        tiles.len(),
        num_threads
    );

    thread::scope(|scope| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            let tile_index = Arc::clone(&tile_index);
            let pixels = Arc::clone(&pixels);
            let sender = sender.clone();
            let mut sampler = sampler.clone();

            handles.push(scope.spawn(move || loop {
                let index = tile_index.fetch_add(1, Ordering::SeqCst);
                if index >= tiles.len() {
                    break;
                }

                render_tile(&mut sampler, &tiles[index], scene, width, &pixels);

                update_render_progress(
                    start,
                    tile_index.load(Ordering::Relaxed).min(tiles.len()),
                    tiles.len(),
                );

                if sender.send(index).is_err() {
                    // The receiver has early exited
                    break;
                }
            }));
        }

        if preview {
            let mut window = create_preview_window(width, height);
            show_preview(
                &mut window,
                width,
                height,
                tile_width,
                tile_height,
                tiles,
                Arc::clone(&pixels),
                receiver,
                |x, y| {
                    info!("Rendering pixel at ({x},{y})");
                    let mut color = Color::BLACK;
                    for sample_index in 0..sampler.num_samples() {
                        color += render_pixel(&mut sampler, x, y, sample_index, scene);
                    }
                    color /= sampler.num_samples() as f64;
                    info!("Color = {} {:?}", color, color.to_rgb());
                },
                on_finish,
            );
        } else {
            for handle in handles {
                handle.join().unwrap();
            }
            on_finish();
        }
    });
}

#[derive(Parser)]
struct Cli {
    #[clap(long, short)]
    scene: String,

    #[clap(long, default_value_t = String::from("out.exr"))]
    output: String,

    #[clap(long)]
    preview: bool,

    #[clap(long, default_value_t = 0)]
    seed: usize,
}

fn main() -> Result<(), ParserError> {
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Cli::parse();

    let start = Instant::now();
    let input = std::fs::read_to_string(&args.scene).expect("Error reading scene file");
    let scene = match parse_scene(&input) {
        Ok(scene) => scene,
        Err(e) => {
            match e.location {
                Some(location) => error!("{} at {}:{}", e.message, args.scene, location),
                None => error!("{} in {}", e.message, args.scene),
            }
            return Ok(());
        }
    };
    info!("Scene constructed in {:?}", start.elapsed());

    let (width, height) = scene.film_bounds();

    // Render to a buffer
    let sampler = SobolSampler::new(args.seed, scene.num_samples);
    render(&scene, sampler, args.preview, start, |pixels| {
        eprintln!();
        info!("Rendering finished in {:.1?}", start.elapsed());

        // Save to file
        let image_buffer =
            image::Rgb32FImage::from_raw(width as u32, height as u32, pixels).unwrap();
        image_buffer.save(&args.output).expect("Error saving file");
        info!("Output written to {}", &args.output);
    });

    Ok(())
}
