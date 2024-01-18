use std::sync::Arc;

use crate::{
    bvh::{Bvh, SplitMethod},
    camera::Camera,
    intersection::PrimitiveIntersection,
    light::Light,
    primitive::Primitive,
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub struct Scene {
    pub max_depth: usize,
    pub num_samples: usize,
    pub camera: Camera,
    pub lights: Vec<Arc<Light>>,
    bvh: Bvh,
}

impl Scene {
    pub fn new(
        max_depth: usize,
        num_samples: usize,
        camera: Camera,
        lights: Vec<Arc<Light>>,
        primitives: Vec<Arc<Primitive>>,
    ) -> Self {
        let bvh = Bvh::new(primitives, SplitMethod::Median);
        Self {
            max_depth,
            num_samples,
            camera,
            lights,
            bvh,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        self.bvh.intersect(ray)
    }

    pub fn film_bounds(&self) -> (usize, usize) {
        (self.camera.film.width, self.camera.film.height)
    }
}
