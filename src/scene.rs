use std::sync::Arc;

use crate::{
    bvh::BvhNode, camera::Camera, intersection::PrimitiveIntersection, light::Light,
    primitive::Primitive, ray::Ray,
};

#[derive(Debug, PartialEq)]
pub struct Scene {
    pub max_depth: usize,
    pub num_samples: usize,
    pub film_width: usize,
    pub film_height: usize,
    pub camera: Camera,
    pub lights: Vec<Light>,
    bvh: BvhNode,
}

impl Scene {
    pub fn new(
        max_depth: usize,
        num_samples: usize,
        film_width: usize,
        film_height: usize,
        camera: Camera,
        lights: Vec<Light>,
        primitives: Vec<Arc<Primitive>>,
    ) -> Self {
        Self {
            max_depth,
            num_samples,
            film_width,
            film_height,
            camera,
            lights,
            bvh: BvhNode::new(primitives),
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        self.bvh.intersect(ray)
    }
}
