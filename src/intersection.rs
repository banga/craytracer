use crate::{material::Material, vector::Vector};

pub struct ShapeIntersection {
    pub location: Vector,
    // TODO: Get the normal lazily when needed
    pub normal: Vector,
}

pub struct PrimitiveIntersection<'a> {
    pub distance: f64,
    pub location: Vector,
    pub normal: Vector,
    pub material: &'a dyn Material,
}
