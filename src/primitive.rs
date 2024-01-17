use std::sync::Arc;

use crate::{
    bounds::Bounds, intersection::PrimitiveIntersection, light::Light, material::Material,
    ray::Ray, shape::Shape,
};

#[derive(Debug, PartialEq)]
pub enum Primitive {
    ShapePrimitive {
        shape: Arc<Shape>,
        material: Arc<Material>,
        area_light: Option<Arc<Light>>,
    },
}

impl Primitive {
    pub fn new(shape: Arc<Shape>, material: Arc<Material>, area_light: Option<Arc<Light>>) -> Self {
        if let Some(area_light) = &area_light {
            assert!(
                matches!(**area_light, Light::Area { .. }),
                "Non area light provided as area light for shape"
            );
        }

        Self::ShapePrimitive {
            shape,
            material,
            area_light,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        match self {
            Primitive::ShapePrimitive {
                shape, material, ..
            } => {
                let intersection = shape.intersect(ray)?;
                Some(PrimitiveIntersection {
                    distance: ray.max_distance,
                    normal: intersection.normal,
                    location: intersection.location,
                    material,
                    primitive: self,
                })
            }
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Primitive::ShapePrimitive { shape, .. } => shape.bounds(),
        }
    }

    pub fn get_area_light(&self) -> &Option<Arc<Light>> {
        match self {
            Primitive::ShapePrimitive { area_light, .. } => area_light,
        }
    }
}
