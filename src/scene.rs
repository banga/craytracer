use std::{sync::Arc, time::Instant};

use log::debug;

use crate::{
    bvh::{Bvh, SplitMethod},
    camera::Camera,
    intersection::PrimitiveIntersection,
    light::{Light, LightSampler},
    primitive::Primitive,
    ray::Ray,
};

#[derive(Debug)]
pub struct Scene {
    pub max_depth: usize,
    pub num_samples: usize,
    pub camera: Camera,
    pub lights: Vec<Arc<Light>>,
    pub light_sampler: LightSampler,
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
        // TODO: Maybe allow picking split method in scene files
        let start = Instant::now();
        debug!(
            "Scene with {} lights and {} primitives",
            lights.len(),
            primitives.len()
        );
        let bvh = Bvh::new(primitives, SplitMethod::SAH);
        debug!("BVH constructed in {:?}", start.elapsed());

        let world_radius = bvh.bounds.diagonal().magnitude() * 0.5;
        let light_sampler = LightSampler::new(&lights, world_radius);

        Self {
            max_depth,
            num_samples,
            camera,
            lights,
            light_sampler,
            bvh,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        self.bvh.intersect(ray)
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        self.bvh.intersects(ray)
    }

    pub fn film_bounds(&self) -> (usize, usize) {
        (self.camera.film.width, self.camera.film.height)
    }
}
