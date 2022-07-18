use indicatif::{ProgressBar, ProgressStyle};
use scene::Scene;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use trace::trace;

use crate::image::Image;

mod camera;
mod constants;
mod scenes;
mod image;
mod intersection;
mod material;
mod ray;
mod sampling;
mod scene;
mod shape;
mod trace;
mod vector;

fn render(scene: Scene) {
    let width = scene.film_width;
    let height = scene.film_height;

    let image = Arc::new(Mutex::new(Image::new(width, height)));
    let pool = ThreadPool::new(num_cpus::get());
    let num_pixels = (width * height) as u64;
    let num_pixels_traced = Arc::new(Mutex::new(0));

    // Rendering
    let scene = Arc::new(scene);
    for y in 0..height {
        let image = Arc::clone(&image);
        let scene = Arc::clone(&scene);
        let num_pixels_traced = Arc::clone(&num_pixels_traced);
        pool.execute(move || {
            for x in 0..width {
                let color = scene.camera.sample(x, y, &scene);
                image.lock().unwrap().set_pixel(x, y, color);
                *(num_pixels_traced.lock().unwrap()) += 1;
            }
        });
    }

    // Progress bar
    let pb = ProgressBar::new(num_pixels);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {pos}/{len} pixels {wide_bar} [{elapsed} / {duration}]"),
    );
    loop {
        if let Ok(num_pixels_traced) = num_pixels_traced.try_lock() {
            pb.set_position(*num_pixels_traced);
            if *num_pixels_traced == num_pixels {
                pb.finish();
                break;
            }
        }
        pb.set_message(format!(
            "{}x{}x{} {}/{} threads",
            width,
            height,
            scene.max_depth,
            pool.active_count(),
            pool.max_count()
        ));
        thread::sleep(Duration::from_millis(200));
    }

    pool.join();

    image
        .lock()
        .unwrap()
        .write("out.ppm")
        .expect("Error writing image");
}

fn main() {
    let scene = scenes::logo();
    render(scene);
}
