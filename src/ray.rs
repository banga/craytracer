use crate::vector::{Vector};

pub struct Ray {
    origin: Vector,
    direction: Vector
}

impl Ray {
    pub fn new(origin: Vector, direction: Vector) -> Ray {
        // assert_eq!(1.0, direction.magnitude());
        Ray { origin, direction }
    }
    pub fn origin<'a>(&'a self) -> &'a Vector {
        &self.origin
    }
    pub fn direction<'a>(&'a self) -> &'a Vector {
        &self.direction
    }
    pub fn at(&self, t: f64) -> Vector {
        self.origin + self.direction * t
    }
}