use crate::{bvh::BvhNode, camera::Camera};

pub struct Scene {
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<dyn Camera>,
    pub bvh: Box<BvhNode>,
    pub max_depth: u32,
}
