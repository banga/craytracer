use crate::{
    constants::EPSILON, intersection::Intersection, material::Material, ray::Ray, vector::Vector,
};

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

        let mut distance = (-b - discriminant.sqrt()) / (2.0 * a);
        if distance < EPSILON {
            distance = (-b + discriminant.sqrt()) / (2.0 * a);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{color::Color, material::LambertianMaterial};

    #[test]
    fn intersect() {
        // Unit sphere at origin
        let sphere = Sphere {
            origin: Vector(0.0, 0.0, 0.0),
            radius: 1.0,
            material: Box::new(LambertianMaterial {
                reflectance: Color::WHITE,
                num_samples: 1,
            }),
        };

        // Shoot ray from outside sphere
        let intersection = sphere
            .intersect(&Ray {
                origin: Vector(0.0, 0.0, -2.0),
                direction: Vector::Z,
            })
            .unwrap();
        assert_eq!(intersection.location, Vector(0.0, 0.0, -1.0));
        assert_eq!(intersection.normal, Vector(0.0, 0.0, -1.0));

        // Shoot ray from inside sphere
        let intersection = sphere
            .intersect(&Ray {
                origin: Vector(0.0, 0.0, 0.0),
                direction: Vector::Z,
            })
            .unwrap();
        assert_eq!(intersection.location, Vector(0.0, 0.0, 1.0));
        assert_eq!(intersection.normal, Vector(0.0, 0.0, 1.0));

        // Shoot ray away from sphere
        assert!(sphere
            .intersect(&Ray {
                origin: Vector(0.0, 0.0, -2.0),
                direction: Vector::X,
            })
            .is_none());
    }
}
