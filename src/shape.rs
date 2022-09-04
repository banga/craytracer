use crate::{
    bounds::Bounds, constants::EPSILON, intersection::ShapeIntersection, ray::Ray, vector::Vector,
};

pub trait Shape: Sync + Send {
    // Should update the ray's max_distance if an intersection is found
    fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection>;
    fn bounds(&self) -> Bounds;
}

pub struct Sphere {
    pub origin: Vector,
    pub radius: f64,
}

impl Shape for Sphere {
    fn bounds(&self) -> Bounds {
        Bounds::new(
            Vector(
                self.origin.x() - self.radius,
                self.origin.y() - self.radius,
                self.origin.z() - self.radius,
            ),
            Vector(
                self.origin.x() + self.radius,
                self.origin.y() + self.radius,
                self.origin.z() + self.radius,
            ),
        )
    }

    fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection> {
        let oc = ray.origin - self.origin;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let mut distance = (-b - discriminant.sqrt()) / (2.0 * a);
        if let Some(location) = ray.update_max_distance(distance) {
            return Some(ShapeIntersection {
                location,
                normal: (location - self.origin) / self.radius,
            });
        }

        distance = (-b + discriminant.sqrt()) / (2.0 * a);
        if let Some(location) = ray.update_max_distance(distance) {
            return Some(ShapeIntersection {
                location,
                normal: (location - self.origin) / self.radius,
            });
        }

        None
    }
}

pub struct Triangle {
    v0: Vector,
    e1: Vector,
    e2: Vector,
    n0: Vector,
    n01: Vector,
    n02: Vector,
}

impl Triangle {
    pub fn new(v0: Vector, v1: Vector, v2: Vector) -> Triangle {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        let n0 = e1.cross(&e2).normalized();

        Triangle {
            v0,
            e1,
            e2,
            n0,
            n01: Vector::O,
            n02: Vector::O,
        }
    }
    pub fn with_normals(
        v0: Vector,
        v1: Vector,
        v2: Vector,
        n0: Vector,
        n1: Vector,
        n2: Vector,
    ) -> Triangle {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        Triangle {
            v0,
            e1,
            e2,
            n0,
            n01: n1 - n0,
            n02: n2 - n0,
        }
    }
}

impl Shape for Triangle {
    fn bounds(&self) -> Bounds {
        let v1 = self.v0 + self.e1;
        let v2 = self.v0 + self.e2;

        Bounds::new(
            Vector(
                v1.x().min(v2.x().min(self.v0.x())),
                v1.y().min(v2.y().min(self.v0.y())),
                v1.z().min(v2.z().min(self.v0.z())),
            ),
            Vector(
                v1.x().max(v2.x().max(self.v0.x())),
                v1.y().max(v2.y().max(self.v0.y())),
                v1.z().max(v2.z().max(self.v0.z())),
            ),
        )
    }

    #[allow(non_snake_case)]
    fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection> {
        // Source: http://www.graphics.cornell.edu/pubs/1997/MT97.pdf
        let P = ray.direction.cross(&self.e2);

        let denominator = P.dot(&self.e1);
        if denominator > -EPSILON && denominator < EPSILON {
            return None;
        }

        let T = ray.origin - self.v0;
        let inv_denominator = 1.0 / denominator;
        let u = P.dot(&T) * inv_denominator;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let Q = T.cross(&self.e1);
        let v = Q.dot(&ray.direction) * inv_denominator;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let distance = T.cross(&self.e1).dot(&self.e2) * inv_denominator;
        if let Some(location) = ray.update_max_distance(distance) {
            Some(ShapeIntersection {
                location,
                normal: (self.n0 + self.n01 * u + self.n02 * v).normalized(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sphere {
        use super::*;

        #[test]
        fn bounds() {
            assert_eq!(
                Sphere {
                    origin: Vector(0.0, 0.0, 0.0),
                    radius: 1.0,
                }
                .bounds(),
                Bounds::new(Vector(-1.0, -1.0, -1.0), Vector(1.0, 1.0, 1.0),)
            );

            assert_eq!(
                Sphere {
                    origin: Vector(-2.0, 3.0, 0.0),
                    radius: 1.0,
                }
                .bounds(),
                Bounds::new(Vector(-3.0, 2.0, -1.0), Vector(-1.0, 4.0, 1.0),)
            );
        }
    }

    mod triangle {
        use approx::assert_abs_diff_eq;
        use rand::{thread_rng, Rng};

        use super::*;

        // Triangle in XY plane
        fn triangle() -> Triangle {
            Triangle::new(
                Vector(1.0, 0.0, 0.0),
                Vector(1.0, 1.0, 0.0),
                Vector(2.0, 0.0, 0.0),
            )
        }

        #[test]
        fn bounds() {
            assert_eq!(
                triangle().bounds(),
                Bounds::new(Vector(1.0, 0.0, 0.0), Vector(2.0, 1.0, 0.0),)
            );
        }

        #[test]
        fn intersect_vertices() {
            // Shoot ray to hit v0
            let t = triangle();
            for point in [t.v0, t.v0 + t.e1, t.v0 + t.e2] {
                let ray = &mut Ray::new(Vector(point.x(), point.y(), -2.0), Vector::Z);
                let intersection = t.intersect(ray).unwrap();
                assert_eq!(ray.max_distance, 2.0);
                assert_eq!(intersection.normal, Vector(0.0, 0.0, -1.0));
            }
        }

        #[test]
        fn from_behind() {
            // Shoot ray from the opposite side
            let t = triangle();
            let ray = &mut Ray::new(Vector(1.0, 0.0, 2.0), -Vector::Z);
            let intersection = t.intersect(ray).unwrap();
            assert_eq!(ray.max_distance, 2.0);
            assert_eq!(intersection.normal, Vector(0.0, 0.0, -1.0));
        }

        #[test]
        fn parallel_to_triangle() {
            assert!(triangle()
                .intersect(&mut Ray::new(
                    Vector(0.0, 0.0, 0.0),
                    Vector(1.0, 1.0, 0.0).normalized(),
                ))
                .is_none());
        }

        #[test]
        fn random_point() {
            let t = triangle();

            let mut rng = thread_rng();
            let u = rng.gen_range(0.0..1.0);
            let v = rng.gen_range(0.0..1.0);
            let target = t.v0 + t.e1 * u + t.e2 * v;

            let origin = Vector(0.0, 0.0, -2.0);
            let ray = &mut Ray::new(origin, (target - origin).normalized());
            let intersection = t.intersect(ray);

            if u + v <= 1.0 {
                let intersection = intersection.expect("Expected an intersection");
                assert_abs_diff_eq!(
                    ray.max_distance,
                    (target - origin).magnitude(),
                    epsilon = EPSILON
                );
                assert_eq!(intersection.normal, Vector(0.0, 0.0, -1.0));
            } else {
                assert!(intersection.is_none());
            }
        }
    }
}
