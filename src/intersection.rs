use crate::{
    geometry::{point::Point, vector::Vector},
    material::Material,
};

pub struct ShapeIntersection {
    pub location: Point,
    // TODO: Get the normal lazily when needed
    pub normal: Vector,
}

pub struct PrimitiveIntersection<'a> {
    pub distance: f64,
    pub location: Point,
    pub normal: Vector,
    pub material: &'a Material,
}
