use crate::{intersection::Intersection, ray::Ray, vector::Vector, constants::EPSILON};

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

pub struct Sphere {
    origin: Vector,
    radius: f64,
}

impl Sphere {
    pub fn new(origin: Vector, radius: f64) -> Sphere {
        Sphere { origin, radius }
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let oc = *ray.origin() - self.origin;
        let a = ray.direction().dot(ray.direction());
        let b = 2.0 * oc.dot(ray.direction());
        let c = oc.dot(&oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let distance = (-b - discriminant.sqrt()) / (2.0 * a);
        if distance < EPSILON {
            return None;
        }
        
        let location = ray.at(distance);
        let normal = (location - self.origin) / self.radius;
        Some(Intersection {
            distance,
            location,
            normal,
        })
    }
}
