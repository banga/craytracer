use std::sync::Arc;

use crate::{
    bounds::Bounds, intersection::PrimitiveIntersection, material::Material, ray::Ray, shape::Shape,
};

pub enum Primitive {
    ShapePrimitive {
        shape: Arc<Shape>,
        material: Arc<Material>,
    },
}

impl Primitive {
    pub fn new_shape_primitive(shape: Arc<Shape>, material: Arc<Material>) -> Self {
        Self::ShapePrimitive { shape, material }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        match self {
            Primitive::ShapePrimitive { shape, material } => {
                let intersection = shape.intersect(ray)?;
                Some(PrimitiveIntersection {
                    distance: ray.max_distance,
                    normal: intersection.normal,
                    location: intersection.location,
                    material,
                })
            }
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Primitive::ShapePrimitive { shape, .. } => shape.bounds(),
        }
    }
}
