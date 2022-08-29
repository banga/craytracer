use crate::{intersection::Intersection, material::Material, ray::Ray, shape::Shape};

pub trait Primitive: Sync + Send {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn material(&self) -> &Box<dyn Material>;
}

pub struct ShapePrimitive {
    pub shape: Box<dyn Shape>,
    pub material: Box<dyn Material>,
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

    fn material(&self) -> &Box<dyn Material> {
        &self.material
    }
}
