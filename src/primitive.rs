use std::sync::Arc;

use crate::{
    bounds::Bounds, intersection::PrimitiveIntersection, material::Material, ray::Ray, shape::Shape,
};

pub trait Primitive: Sync + Send {
    fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection>;
    fn material(&self) -> Arc<dyn Material>;
    fn bounds(&self) -> Bounds;
}

pub struct ShapePrimitive {
    pub shape: Box<dyn Shape>,
    pub material: Arc<dyn Material>,
}

impl Primitive for ShapePrimitive {
    fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        let intersection = self.shape.intersect(ray)?;
        Some(PrimitiveIntersection {
            distance: ray.max_distance,
            normal: intersection.normal,
            location: intersection.location,
            material: &*self.material,
        })
    }

    fn material(&self) -> Arc<dyn Material> {
        Arc::clone(&self.material)
    }

    fn bounds(&self) -> Bounds {
        self.shape.bounds()
    }
}
