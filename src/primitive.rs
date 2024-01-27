use std::sync::Arc;

use crate::{
    bounds::Bounds,
    color::Color,
    intersection::{PrimitiveIntersection, ShapeIntersection},
    light::Light,
    material::Material,
    ray::Ray,
    shape::Shape,
};

#[derive(Debug, PartialEq)]
pub enum Primitive {
    ShapePrimitive {
        shape: Arc<Shape>,
        material: Arc<Material>,
    },
    AreaLightPrimitive {
        shape: Arc<Shape>,
        material: Arc<Material>,
        area_light: Arc<Light>,
    },
}

impl Primitive {
    pub fn new(shape: Arc<Shape>, material: Arc<Material>) -> Self {
        Self::ShapePrimitive { shape, material }
    }

    pub fn new_area_light(shape: Arc<Shape>, area_light: Arc<Light>) -> Self {
        assert!(
            matches!(*area_light, Light::Area { .. }),
            "Non area light provided as area light for shape"
        );
        Self::AreaLightPrimitive {
            shape,
            area_light,
            // Set the material to black so that paths will terminate at area
            // lights. Allowing the paths to continue can cause bad results when
            // the material samples the light itself.
            material: Arc::new(Material::new_matte(Color::BLACK, 0.0)),
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        let (shape, material) = match self {
            Primitive::ShapePrimitive {
                shape, material, ..
            } => (shape, material),
            Primitive::AreaLightPrimitive {
                shape, material, ..
            } => (shape, material),
        };

        let ShapeIntersection { location, normal } = shape.intersect(ray)?;
        Some(PrimitiveIntersection {
            distance: ray.max_distance,
            normal,
            location,
            material,
            primitive: self,
        })
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        let shape = match self {
            Primitive::ShapePrimitive { shape, .. } => shape,
            Primitive::AreaLightPrimitive { shape, .. } => shape,
        };

        shape.intersects(ray)
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Primitive::AreaLightPrimitive { shape, .. } => shape,
            Primitive::ShapePrimitive { shape, .. } => shape,
        }
        .bounds()
    }

    pub fn get_area_light(&self) -> Option<&Arc<Light>> {
        match self {
            Primitive::ShapePrimitive { .. } => None,
            Primitive::AreaLightPrimitive { area_light, .. } => Some(area_light),
        }
    }
}
