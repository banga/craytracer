use crate::{bounds::Bounds, constants::EPSILON, ray::Ray, vector::Vector};

pub trait Shape: Sync + Send {
    // Returns distance and normal
    fn intersect(&self, ray: &Ray) -> Option<(f64, Vector)>;
    fn bounds(&self) -> Bounds;
}

pub struct Sphere {
    pub origin: Vector,
    pub radius: f64,
}

impl Shape for Sphere {
    fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector(
                self.origin.x() - self.radius,
                self.origin.y() - self.radius,
                self.origin.z() - self.radius,
            ),
            max: Vector(
                self.origin.x() + self.radius,
                self.origin.y() + self.radius,
                self.origin.z() + self.radius,
            ),
        }
    }

    fn intersect(&self, ray: &Ray) -> Option<(f64, Vector)> {
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

        Some((distance, normal))
    }
}

pub struct Triangle {
    v0: Vector,
    e1: Vector,
    e2: Vector,
    normal: Vector,
}

impl Triangle {
    pub fn new(v0: Vector, v1: Vector, v2: Vector) -> Triangle {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        Triangle {
            v0,
            e1,
            e2,
            // Arbitrary choice, may need to be flipped for some meshes
            normal: e1.cross(&e2).normalized(),
        }
    }
}

impl Shape for Triangle {
    fn bounds(&self) -> Bounds {
        let v1 = self.v0 + self.e1;
        let v2 = self.v0 + self.e2;

        Bounds {
            min: Vector(
                v1.x().min(v2.x().min(self.v0.x())),
                v1.y().min(v2.y().min(self.v0.y())),
                v1.z().min(v2.z().min(self.v0.z())),
            ),
            max: Vector(
                v1.x().max(v2.x().max(self.v0.x())),
                v1.y().max(v2.y().max(self.v0.y())),
                v1.z().max(v2.z().max(self.v0.z())),
            ),
        }
    }

    #[allow(non_snake_case)]
    fn intersect(&self, ray: &Ray) -> Option<(f64, Vector)> {
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
        if distance < EPSILON {
            return None;
        }

        Some((distance, self.normal))
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
                Bounds {
                    min: Vector(-1.0, -1.0, -1.0),
                    max: Vector(1.0, 1.0, 1.0),
                }
            );

            assert_eq!(
                Sphere {
                    origin: Vector(-2.0, 3.0, 0.0),
                    radius: 1.0,
                }
                .bounds(),
                Bounds {
                    min: Vector(-3.0, 2.0, -1.0),
                    max: Vector(-1.0, 4.0, 1.0),
                }
            );
        }

        #[test]
        fn intersect() {
            // Unit sphere at origin
            let sphere = Sphere {
                origin: Vector(0.0, 0.0, 0.0),
                radius: 1.0,
            };

            // Shoot ray from outside sphere
            let (distance, normal) = sphere
                .intersect(&Ray {
                    origin: Vector(0.0, 0.0, -2.0),
                    direction: Vector::Z,
                })
                .unwrap();
            assert_eq!(distance, 1.0);
            assert_eq!(normal, Vector(0.0, 0.0, -1.0));

            // Shoot ray from inside sphere
            let (distance, normal) = sphere
                .intersect(&Ray {
                    origin: Vector(0.0, 0.0, 0.0),
                    direction: Vector::Z,
                })
                .unwrap();
            assert_eq!(distance, 1.0);
            assert_eq!(normal, Vector(0.0, 0.0, 1.0));

            // Shoot ray away from sphere
            assert!(sphere
                .intersect(&Ray {
                    origin: Vector(0.0, 0.0, -2.0),
                    direction: Vector::X,
                })
                .is_none());
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
                Bounds {
                    min: Vector(1.0, 0.0, 0.0),
                    max: Vector(2.0, 1.0, 0.0),
                }
            );
        }

        #[test]
        fn intersect_vertices() {
            // Shoot ray to hit v0
            let t = triangle();
            for point in [t.v0, t.v0 + t.e1, t.v0 + t.e2] {
                let (distance, normal) = t
                    .intersect(&Ray {
                        origin: Vector(point.x(), point.y(), -2.0),
                        direction: Vector::Z,
                    })
                    .unwrap();
                assert_eq!(distance, 2.0);
                assert_eq!(normal, Vector(0.0, 0.0, -1.0));
            }
        }

        #[test]
        fn from_behind() {
            // Shoot ray from the opposite side
            let t = triangle();
            let (distance, normal) = t
                .intersect(&Ray {
                    origin: Vector(1.0, 0.0, 2.0),
                    direction: -Vector::Z,
                })
                .unwrap();
            assert_eq!(distance, 2.0);
            assert_eq!(normal, Vector(0.0, 0.0, -1.0));
        }

        #[test]
        fn parallel_to_triangle() {
            assert!(triangle()
                .intersect(&Ray {
                    origin: Vector(0.0, 0.0, 0.0),
                    direction: Vector(1.0, 1.0, 0.0).normalized(),
                })
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
            let intersection = t.intersect(&Ray {
                origin,
                direction: (target - origin).normalized(),
            });

            if u + v <= 1.0 {
                let (distance, normal) = intersection.expect("Expected an intersection");
                assert_abs_diff_eq!(distance, (target - origin).magnitude(), epsilon = EPSILON);
                assert_eq!(normal, Vector(0.0, 0.0, -1.0));
            } else {
                assert!(intersection.is_none());
            }
        }
    }
}
