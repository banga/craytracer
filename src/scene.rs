use crate::{camera::Camera, shape::Shape};

pub struct Scene {
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<dyn Camera>,
    pub shapes: Vec<Box<dyn Shape>>,
    pub max_depth: u32,
}
