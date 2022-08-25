use crate::{camera::Camera, color::Color, shape::Shape};

pub struct Scene {
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<dyn Camera>,
    pub shapes: Vec<Box<dyn Shape>>,
    pub background: Color,
    pub max_depth: u32,
}
