use crate::{shape::Shape, vector::Vector};

pub struct Intersection<'a> {
    pub distance: f64,
    pub location: Vector,
    pub normal: Vector,
    pub shape: &'a dyn Shape,
}
