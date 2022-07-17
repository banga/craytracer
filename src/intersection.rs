use crate::{vector::Vector, shape::Shape};

pub struct Intersection<'a> {
    pub distance: f64,
    pub location: Vector,
    pub normal: Vector,
    pub shape: &'a dyn Shape,
}
