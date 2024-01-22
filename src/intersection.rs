use crate::{
    color::Color,
    geometry::{normal::Normal, point::Point, vector::Vector},
    material::Material,
    primitive::Primitive,
};

#[derive(Debug, PartialEq)]
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
    pub primitive: &'a Primitive,
}

impl<'a> PrimitiveIntersection<'a> {
    /// Light emitted at the current intersection point in the given direction
    #[allow(non_snake_case)]
    pub fn Le(&self, w_o: &Vector) -> Color {
        match self.primitive.get_area_light() {
            Some(area_light) => area_light.L(self, &w_o),
            None => Color::BLACK,
        }
    }
}
