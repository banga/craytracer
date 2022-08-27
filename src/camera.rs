use crate::{color::Color, ray::Ray, sampling::sample_2d, scene::Scene, trace, vector::Vector};

pub trait Camera: Send + Sync {
    fn sample(&self, x: usize, y: usize, scene: &Scene) -> Color;
}

pub struct ProjectionCamera {
    origin: Vector,
    x: Vector,
    y: Vector,
    z: Vector,
    focal_distance: f64,
    num_samples: usize,
    delta_x: f64,
    delta_y: f64,
}

impl ProjectionCamera {
    pub fn new(
        origin: Vector,
        target: Vector,
        up: Vector,
        focal_distance: f64,
        num_samples: usize,
        film_width: usize,
        film_height: usize,
    ) -> ProjectionCamera {
        let z = (target - origin).normalized();
        let x = up.normalized().cross(&z) * (film_width as f64 / film_height as f64);
        let y = z.cross(&x).normalized();

        let delta_x = 1.0 / film_width as f64;
        let delta_y = 1.0 / film_height as f64;

        ProjectionCamera {
            origin,
            x,
            y,
            z,
            focal_distance,
            num_samples,
            delta_x,
            delta_y,
        }
    }
}

impl Camera for ProjectionCamera {
    fn sample(&self, x: usize, y: usize, scene: &Scene) -> Color {
        // Adjust sx and sy to center the screen space center to the center
        // of the film
        let sx = x as f64 * self.delta_x - 0.5;
        let sy = y as f64 * self.delta_y as f64 - 0.5;
        let ray_origin = self.origin
            + self.z * self.focal_distance
            + self.x * sx
            // Screen space y co-ordinates are flipped, hence the minus sign
            - self.y * sy;

        let mut color = Color::BLACK;
        for _ in 0..self.num_samples {
            let (dx, dy) = sample_2d();
            let ray_origin = ray_origin + self.x * dx * self.delta_x + self.y * dy * self.delta_y;
            let ray = Ray::new(ray_origin, (ray_origin - self.origin).normalized());
            color += trace(&ray, &scene, 0);
        }
        color / self.num_samples as f64
    }
}
