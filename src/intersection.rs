use crate::{
    geometry::{normal::Normal, point::Point},
    material::Material,
};

pub struct ShapeIntersection {
    pub location: Point,
    // TODO: Get the normal lazily when needed
    pub normal: Normal,
}

pub struct PrimitiveIntersection<'a> {
    pub distance: f64,
    pub location: Point,
    pub normal: Normal,
    pub material: &'a Material,
}
