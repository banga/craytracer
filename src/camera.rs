use crate::{color::Color, ray::Ray, sampling::sample_2d, scene::Scene, trace, vector::Vector};

pub enum Camera {
    Projection {
        origin: Vector,
        x: Vector,
        y: Vector,
        z: Vector,
        focal_distance: f64,
        num_samples: usize,
        delta_x: f64,
        delta_y: f64,
    },
}

impl Camera {
    pub fn new_projection_camera(
        origin: Vector,
        target: Vector,
        up: Vector,
        focal_distance: f64,
        num_samples: usize,
        film_width: usize,
        film_height: usize,
    ) -> Camera {
        let z = (target - origin).normalized();
        let x = up.normalized().cross(&z) * (film_width as f64 / film_height as f64);
        let y = z.cross(&x).normalized();

        let delta_x = 1.0 / film_width as f64;
        let delta_y = 1.0 / film_height as f64;

        Camera::Projection {
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

    pub fn sample(&self, sx: usize, sy: usize, scene: &Scene) -> Color {
        match self {
            Camera::Projection {
                origin,
                x,
                y,
                z,
                focal_distance,
                num_samples,
                delta_x,
                delta_y,
            } => {
                // Adjust sx and sy to center the screen space center to the center
                // of the film
                let sx = sx as f64 * delta_x - 0.5;
                let sy = sy as f64 * delta_y - 0.5;
                let ray_origin = *origin + (*z) * *focal_distance
                    // Screen space x and y co-ordinates are flipped (and we use
                    // right-handed co-ordinates)
                    - *x * sx
                    - *y * sy;

                let mut color = Color::BLACK;
                for _ in 0..*num_samples {
                    let (dx, dy) = sample_2d();
                    let ray_origin = ray_origin + *x * dx * *delta_x + *y * dy * *delta_y;
                    let mut ray = Ray::new(ray_origin, (ray_origin - *origin).normalized());
                    color += trace(&mut ray, &scene, 0);
                }
                color / *num_samples as f64
            }
        }
    }
}
