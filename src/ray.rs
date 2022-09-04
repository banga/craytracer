use approx::assert_abs_diff_eq;

use crate::{constants::EPSILON, vector::Vector};

pub struct Ray {
    pub origin: Vector,
    pub direction: Vector,
    pub max_distance: f64,
}

impl Ray {
    pub fn new(origin: Vector, direction: Vector) -> Ray {
        assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);
        Ray {
            origin,
            direction,
            max_distance: f64::INFINITY,
        }
    }
    pub fn update_max_distance(&mut self, distance: f64) -> Option<Vector> {
        if distance > EPSILON && distance < self.max_distance {
            self.max_distance = distance;
            Some(self.origin + self.direction * distance)
        } else {
            None
        }
    }
}
