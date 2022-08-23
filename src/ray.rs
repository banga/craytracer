use approx::assert_abs_diff_eq;

use crate::{vector::Vector, constants::EPSILON};

pub struct Ray {
    pub origin: Vector,
    pub direction: Vector,
}

impl Ray {
    pub fn new(origin: Vector, direction: Vector) -> Ray {
        assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon=EPSILON);
        Ray { origin, direction }
    }
    pub fn at(&self, t: f64) -> Vector {
        self.origin + self.direction * t
    }
}
