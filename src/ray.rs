use crate::vector::{Vector};

pub struct Ray {
    pub origin: Vector,
    pub direction: Vector
}

impl Ray {
    pub fn new(origin: Vector, direction: Vector) -> Ray {
        // assert_eq!(1.0, direction.magnitude());
        Ray { origin, direction }
    }
    pub fn at(&self, t: f64) -> Vector {
        self.origin + self.direction * t
    }
}