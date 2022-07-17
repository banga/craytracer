use std::cmp::Ordering;

use camera::{Camera, ProjectionCamera};
use intersection::Intersection;
use ray::Ray;
use shape::{Shape, Sphere};

use crate::image::Image;
use crate::vector::{Color, Vector};

mod camera;
mod image;
mod intersection;
mod ray;
mod shape;
mod vector;

fn sky(ray: &Ray) -> Color {
    let t = (ray.direction().y() + 1.0) * 0.5;
    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

fn intersect(ray: &Ray, shapes: &Vec<Sphere>) -> Option<Intersection> {
    let infinite = Intersection {
        distance: f64::INFINITY,
        location: Vector::NULL,
        normal: Vector::NULL,
    };

    shapes
        .iter()
        .map(|shape| shape.intersect(ray))
        .min_by(|a, b| {
            if a.as_ref().unwrap_or(&infinite).distance < b.as_ref().unwrap_or(&infinite).distance {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .expect("Expected to find a single intersection result, did you provide any shapes?")
}

fn main() {
    let mut image = Image::new(1360, 800);

    let camera = ProjectionCamera::new(
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, 0.0),
        Vector::Y,
        1.0,
        image.width as f64 / image.height as f64,
    );
    let shapes = vec![
        // Ground
        Sphere::new(Vector::new(0.0, 0.0, 10.0), 1.0),
        Sphere::new(Vector::new(0.0, -51.0, 10.0), 50.0),
    ];

    for x in 0..image.width {
        let sx = x as f64 / image.width as f64;
        for y in 0..image.height {
            let sy = y as f64 / image.height as f64;
            let ray = camera.make_ray(sx, sy);

            // TODO: Lights
            if let Some(intersection) = intersect(&ray, &shapes) {
                let cos_theta = (-ray.direction().dot(&intersection.normal)).clamp(0.0, 1.0);
                image.set_pixel(x, y, Vector::new(1.0, 1.0, 1.0) * cos_theta);
            } else {
                image.set_pixel(x, y, sky(&ray));
            }
        }
    }

    image.write("out.ppm").expect("Error writing image")
}
