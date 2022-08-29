use crate::{camera::Camera, primitive::Primitive};

pub struct Scene {
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<dyn Camera>,
    pub primitives: Vec<Box<dyn Primitive>>,
    pub max_depth: u32,
}
