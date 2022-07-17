use indicatif::{ProgressBar, ProgressStyle};
use rand::prelude::*;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

use camera::{Camera, ProjectionCamera};
use intersection::Intersection;
use ray::Ray;
use shape::{Shape, Sphere};

use crate::image::Image;
use crate::vector::{Color, Vector};

mod camera;
mod constants;
mod image;
mod intersection;
mod ray;
mod shape;
mod vector;

const MAX_DEPTH: u32 = 3;
const RADIANCE_SAMPLES: u32 = 16;
const GAMMA: f64 = 1.0 / 2.0;
const IMAGE_WIDTH: usize = 800;
const IMAGE_HEIGHT: usize = 600;

fn sky(ray: &Ray) -> Color {
    let t = (ray.direction().y() + 1.0) * 0.5;
    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

fn sample_hemisphere(rng: &mut ThreadRng, normal: &Vector) -> Vector {
    loop {
        let v = Vector::new(
            rng.gen::<f64>() * 2.0 - 1.0,
            rng.gen::<f64>() * 2.0 - 1.0,
            rng.gen::<f64>() * 2.0 - 1.0,
        );
        if v.dot(&v) <= 1.0 {
            if v.dot(normal) > 0.0 {
                return v;
            } else {
                return v * -1.0;
            }
        }
    }
}

const INFINITE: Intersection = Intersection {
    distance: f64::INFINITY,
    location: Vector::NULL,
    normal: Vector::NULL,
};

fn intersect(ray: &Ray, shapes: &Vec<Sphere>) -> Option<Intersection> {
    shapes
        .iter()
        .map(|shape| shape.intersect(ray))
        .min_by(|a, b| {
            if a.as_ref().unwrap_or(&INFINITE).distance < b.as_ref().unwrap_or(&INFINITE).distance {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .expect("Expected to find a single intersection result, did you provide any shapes?")
}

fn get_color(
    ray: &Ray,
    shapes: &Vec<Sphere>,
    depth: u32,
    num_samples: u32,
    rng: &mut ThreadRng,
) -> Color {
    // TODO: Lights
    if depth <= 0 {
        return Color::NULL;
    }
    if let Some(intersection) = intersect(&ray, &shapes) {
        let mut color = Color::NULL;
        for _ in 0..num_samples {
            let sample_direction = sample_hemisphere(rng, &intersection.normal);
            let cos_theta = sample_direction.dot(&intersection.normal);
            color += get_color(
                &Ray::new(intersection.location, sample_direction),
                shapes,
                depth - 1,
                num_samples,
                rng,
            ) * cos_theta;
        }
        // Multiplying by 0.5 to emulate 50% reflectance
        (color / num_samples as f64) * 0.5
    } else {
        sky(&ray)
    }
}

fn trace_pixel(
    sx: f64,
    sy: f64,
    camera: &dyn Camera,
    shapes: &Vec<Sphere>,
    rng: &mut ThreadRng,
) -> Color {
    let ray = camera.make_ray(sx, sy);
    get_color(&ray, &shapes, MAX_DEPTH, RADIANCE_SAMPLES, rng).powf(GAMMA)
}

fn main() {
    let num_pixels = (IMAGE_WIDTH * IMAGE_HEIGHT) as u64;

    let image = Arc::new(Mutex::new(Image::new(IMAGE_WIDTH, IMAGE_HEIGHT)));

    let camera = Arc::new(ProjectionCamera::new(
        Vector::new(0.0, 4.0, -10.0),
        Vector::new(0.0, 1.0, 10.0),
        Vector::Y,
        4.0,
        IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64,
    ));
    let shapes = Arc::new(vec![
        // Ground
        Sphere::new(Vector::new(0.0, 1.0, 10.0), 1.0),
        Sphere::new(Vector::new(0.0, -100.0, 10.0), 100.0),
    ]);

    let dx = 1.0 / IMAGE_WIDTH as f64;
    let dy = 1.0 / IMAGE_HEIGHT as f64;

    let pool = Arc::new(ThreadPool::new(num_cpus::get()));
    let num_pixels_traced = Arc::new(Mutex::new(0));

    // Rendering
    for y in 0..IMAGE_HEIGHT {
        let image = Arc::clone(&image);
        let camera = Arc::clone(&camera);
        let shapes = Arc::clone(&shapes);
        let num_pixels_traced = Arc::clone(&num_pixels_traced);
        pool.execute(move || {
            let mut rng = rand::thread_rng();
            let sy = y as f64 * dy;
            for x in 0..IMAGE_WIDTH {
                let sx = x as f64 * dx;
                let color = trace_pixel(sx, sy, &*camera, &*shapes, &mut rng);
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
        pb.set_message(format!("{}/{} threads", pool.active_count(), pool.max_count()));
        thread::sleep(Duration::from_millis(200));
    }

    pool.join();

    image
        .lock()
        .unwrap()
        .write("out.ppm")
        .expect("Error writing image");
}
