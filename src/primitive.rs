use std::sync::Arc;

use crate::{intersection::Intersection, material::Material, ray::Ray, shape::Shape};

pub trait Primitive: Sync + Send {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn material(&self) -> Arc<dyn Material>;
}

pub struct ShapePrimitive {
    pub shape: Box<dyn Shape>,
    pub material: Arc<dyn Material>,
}

impl Primitive for ShapePrimitive {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let (distance, normal) = self.shape.intersect(ray)?;
        Some(Intersection {
            distance,
            normal,
            location: ray.at(distance),
            material: &*self.material,
        })
    }

    fn material(&self) -> Arc<dyn Material> {
        Arc::clone(&self.material)
    }
}
