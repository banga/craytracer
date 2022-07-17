use rand::prelude::*;
use std::cmp::Ordering;

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
const NUM_SAMPLES: u32 = 8;
const GAMMA: f64 = 1.0 / 2.0;

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

fn main() {
    let mut rng = rand::thread_rng();

    let mut image = Image::new(800, 600);

    let camera = ProjectionCamera::new(
        Vector::new(0.0, 4.0, -10.0),
        Vector::new(0.0, 1.0, 10.0),
        Vector::Y,
        4.0,
        image.width as f64 / image.height as f64,
    );
    let shapes = vec![
        // Ground
        Sphere::new(Vector::new(0.0, 1.0, 10.0), 1.0),
        Sphere::new(Vector::new(0.0, -100.0, 10.0), 100.0),
    ];

    for y in 0..image.height {
        let sy = y as f64 / image.height as f64;
        if y % 4 == 0 {
            println!("{}", sy);
        }
        for x in 0..image.width {
            let sx = x as f64 / image.width as f64;
            let ray = camera.make_ray(sx, sy);
            let color = get_color(&ray, &shapes, MAX_DEPTH, NUM_SAMPLES, &mut rng);
            image.set_pixel(x, y, color.powf(GAMMA));
        }
    }

    image.write("out.ppm").expect("Error writing image")
}
