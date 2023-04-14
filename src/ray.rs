use crate::{
    constants::EPSILON,
    geometry::{point::Point, vector::Vector},
};

#[derive(Debug, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
    pub max_distance: f64,
}

impl Ray {
    pub fn new(origin: Point, direction: Vector) -> Ray {
        Ray {
            origin,
            direction,
            max_distance: f64::INFINITY,
        }
    }
    pub fn update_max_distance(&mut self, distance: f64) -> Option<Point> {
        if distance > EPSILON && distance < self.max_distance {
            self.max_distance = distance;
            Some(self.origin + self.direction * distance)
        } else {
            None
        }
    }
}
