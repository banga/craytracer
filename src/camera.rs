use crate::{
    geometry::{point::Point, vector::Vector},
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub enum Camera {
    Projection {
        origin: Point,
        x: Vector,
        y: Vector,
        z: Vector,
        focal_distance: f64,
        film_width: usize,
        film_height: usize,
        aspect_ratio: f64,
    },
}

impl Camera {
    pub fn new_projection_camera(
        origin: Point,
        target: Point,
        up: Vector,
        focal_distance: f64,
        film_width: usize,
        film_height: usize,
    ) -> Camera {
        // We use a left handed co-ordinate system
        let z = (target - origin).normalized();
        let x = up.normalized().cross(&z).normalized();
        let y = z.cross(&x).normalized();

        let aspect_ratio = film_width as f64 / film_height as f64;

        Camera::Projection {
            origin,
            x,
            y,
            z,
            focal_distance,
            film_width,
            film_height,
            aspect_ratio,
        }
    }

    pub fn sample(&self, film_x: f64, film_y: f64) -> Ray {
        match self {
            &Camera::Projection {
                origin,
                x,
                y,
                z,
                focal_distance,
                aspect_ratio,
                ..
            } => {
                let delta =
                    // TODO: The translation by (0.5, 0.5) is a hack to center
                    // the target in the film. We should fix this by introducing
                    // proper transformations.
                    x * (film_x - 0.5) * aspect_ratio + y * (film_y - 0.5) + z * focal_distance;
                let ray_origin = origin + delta;
                return Ray::new(ray_origin, delta.normalized());
            }
        }
    }
}
