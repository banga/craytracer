use indicatif::{ProgressBar, ProgressStyle};
use material::{LambertianMaterial, Mirror};
use scene::Scene;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use trace::trace;

use camera::ProjectionCamera;
use shape::Sphere;

use crate::image::Image;
use crate::vector::Vector;

mod camera;
mod constants;
mod image;
mod intersection;
mod material;
mod ray;
mod sampling;
mod scene;
mod shape;
mod trace;
mod vector;

const MAX_DEPTH: u32 = 3;
const GAMMA: f64 = 1.0 / 2.0;
const IMAGE_WIDTH: usize = 800;
const IMAGE_HEIGHT: usize = 600;

fn main() {
    let num_pixels = (IMAGE_WIDTH * IMAGE_HEIGHT) as u64;

    let image = Arc::new(Mutex::new(Image::new(IMAGE_WIDTH, IMAGE_HEIGHT)));

    let scene = Arc::new(Scene {
        background: Vector(1.0, 1.0, 1.0),
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 4.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64,
        )),
        shapes: vec![
            // Ground
            Box::new(Sphere {
                origin: Vector(1.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Vector(1.0, 1.0, 1.0),
                    num_samples: 32,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-1.0, 0.75, 12.0),
                radius: 0.75,
                material: Box::new(Mirror {}),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -100.0, 10.0),
                radius: 100.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Vector(1.0, 1.0, 1.0),
                    num_samples: 32,
                }),
            }),
        ],
    });

    let dx = 1.0 / IMAGE_WIDTH as f64;
    let dy = 1.0 / IMAGE_HEIGHT as f64;

    let pool = Arc::new(ThreadPool::new(num_cpus::get()));
    let num_pixels_traced = Arc::new(Mutex::new(0));

    // Rendering
    for y in 0..IMAGE_HEIGHT {
        let image = Arc::clone(&image);
        let scene = Arc::clone(&scene);
        let num_pixels_traced = Arc::clone(&num_pixels_traced);
        pool.execute(move || {
            let sy = y as f64 * dy;
            for x in 0..IMAGE_WIDTH {
                let sx = x as f64 * dx;
                let ray = scene.camera.make_ray(sx, sy);
                let color = trace(&ray, &scene, MAX_DEPTH).powf(GAMMA);
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
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            MAX_DEPTH,
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
