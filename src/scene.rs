use crate::{camera::Camera, shape::Shape, vector::Color};

pub struct Scene {
    pub camera: Box<dyn Camera>,
    pub shapes: Vec<Box<dyn Shape>>,
    pub background: Color,
}
