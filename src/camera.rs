use crate::{
    color::Color, ray::Ray, sampling::sample_2d, scene::Scene, trace::trace, vector::Vector,
};

#[derive(Debug, PartialEq)]
pub enum Camera {
    Projection {
        origin: Vector,
        x: Vector,
        y: Vector,
        z: Vector,
        focal_distance: f64,
        // TODO: These things should probably live in the Scene
        num_samples: usize,
        film_width: usize,
        film_height: usize,
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

        Camera::Projection {
            origin,
            x,
            y,
            z,
            focal_distance,
            num_samples,
            film_width,
            film_height,
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
                film_width,
                film_height,
            } => {
                // Adjust sx and sy to center the screen space center to the center
                // of the film
                let sx = sx as f64 / *film_width as f64 - 0.5;
                let sy = sy as f64 / *film_height as f64 - 0.5;
                let ray_origin = *origin + (*z) * *focal_distance
                    // Screen space x and y co-ordinates are flipped (and we use
                    // right-handed co-ordinates)
                    - *x * sx
                    - *y * sy;

                let mut color = Color::BLACK;
                for _ in 0..*num_samples {
                    let (dx, dy) = sample_2d();
                    let ray_origin =
                        ray_origin + *x * dx / *film_width as f64 + *y * dy / *film_height as f64;
                    let mut ray = Ray::new(ray_origin, (ray_origin - *origin).normalized());
                    color += trace(&mut ray, &scene, 0);
                }
                color / *num_samples as f64
            }
        }
    }
}
