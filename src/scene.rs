use std::sync::Arc;

use crate::{
    bvh::BvhNode, camera::Camera, intersection::PrimitiveIntersection, primitive::Primitive,
    ray::Ray,
};

pub struct Scene {
    pub max_depth: u32,
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<dyn Camera>,
    bvh: Box<BvhNode>,
}

impl Scene {
    pub fn new(
        max_depth: u32,
        film_width: usize,
        film_height: usize,
        camera: Box<dyn Camera>,
        primitives: Vec<Arc<dyn Primitive>>,
    ) -> Self {
        Self {
            max_depth,
            film_width,
            film_height,
            camera,
            bvh: BvhNode::new(primitives),
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        self.bvh.intersect(ray)
    }
}
