use crate::{ray::Ray, vector::Vector};

pub trait Camera: Send + Sync {
    /**
    Converts screen space co-ordinates to a Ray in world space.

    Screen space co-ordinates go from (0, 0) to (1, 1)
    where (0, 0) is the top left corner and (1, 1) is
    bottom right.
    */
    fn make_ray(&self, sx: f64, sy: f64) -> Ray;
}

pub struct ProjectionCamera {
    origin: Vector,
    x: Vector,
    y: Vector,
    z: Vector,
    focal_distance: f64,
}

impl ProjectionCamera {
    pub fn new(
        origin: Vector,
        target: Vector,
        up: Vector,
        focal_distance: f64,
        aspect_ratio: f64,
    ) -> ProjectionCamera {
        let z = (target - origin).normalized();
        let x = up.normalized().cross(&z) * aspect_ratio;
        let y = z.cross(&x).normalized();

        ProjectionCamera {
            origin,
            x,
            y,
            z,
            focal_distance,
        }
    }
}

impl Camera for ProjectionCamera {
    fn make_ray(&self, sx: f64, sy: f64) -> Ray {
        let ray_origin =
            self.origin 
            + self.z * self.focal_distance 
            // Adjust sx and sy to center the screen space center to the center
            // of the film
            + self.x * (sx - 0.5) 
            // Screen space y co-ordinates are flipped, hence the minus sign
            - self.y * (sy - 0.5);

        Ray::new(ray_origin, (ray_origin - self.origin).normalized())
    }
}
