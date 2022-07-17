use crate::{intersection::Intersection, material::Material, ray::Ray, vector::Vector};

pub trait Shape: Sync + Send {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn material(&self) -> &Box<dyn Material>;
}

pub struct Sphere {
    pub origin: Vector,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let oc = ray.origin - self.origin;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let distance = (-b - discriminant.sqrt()) / (2.0 * a);
        let location = ray.at(distance);
        let normal = (location - self.origin) / self.radius;

        Some(Intersection {
            distance,
            location,
            normal,
            shape: self,
        })
    }

    fn material(&self) -> &Box<dyn Material> {
        &self.material
    }
}
