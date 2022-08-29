use crate::{material::Material, vector::Vector};

pub struct Intersection<'a> {
    pub distance: f64,
    pub location: Vector,
    pub normal: Vector,
    pub material: &'a dyn Material,
}
