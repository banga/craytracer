use std::sync::Arc;

use crate::{
    bvh::BvhNode, camera::Camera, intersection::PrimitiveIntersection, primitive::Primitive,
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub struct Scene {
    pub max_depth: usize,
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Box<Camera>,
    bvh: Box<BvhNode>,
}

impl Scene {
    pub fn new(
        max_depth: usize,
        film_width: usize,
        film_height: usize,
        camera: Box<Camera>,
        primitives: Vec<Arc<Primitive>>,
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
