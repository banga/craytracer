use crate::{
    bounds::Bounds, constants::EPSILON, intersection::ShapeIntersection, ray::Ray, vector::Vector,
};

pub enum Shape {
    Sphere {
        origin: Vector,
        radius: f64,
        _radius_squared: f64,
        _inv_radius: f64,
    },
    Triangle {
        v0: Vector,
        e1: Vector,
        e2: Vector,
        _n0: Vector,
        _n01: Vector,
        _n02: Vector,
    },
}

impl Shape {
    pub fn new_sphere(origin: Vector, radius: f64) -> Shape {
        Shape::Sphere {
            origin,
            radius,
            _radius_squared: radius * radius,
            _inv_radius: 1.0 / radius,
        }
    }
    pub fn new_triangle(v0: Vector, v1: Vector, v2: Vector) -> Shape {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        let n0 = e1.cross(&e2).normalized();

        Shape::Triangle {
            v0,
            e1,
            e2,
            _n0: n0,
            _n01: Vector::O,
            _n02: Vector::O,
        }
    }
    pub fn new_triangle_with_normals(
        v0: Vector,
        v1: Vector,
        v2: Vector,
        n0: Vector,
        n1: Vector,
        n2: Vector,
    ) -> Shape {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        Shape::Triangle {
            v0,
            e1,
            e2,
            _n0: n0,
            _n01: n1 - n0,
            _n02: n2 - n0,
        }
    }

    #[allow(non_snake_case)]
    // Should update the ray's max_distance if an intersection is found
    pub fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection> {
        match self {
            Shape::Sphere {
                origin,
                radius,
                _radius_squared: radius_squared,
                _inv_radius: inv_radius,
            } => {
                let oc = ray.origin - *origin;
                let a = ray.direction.magnitude_squared();
                let b = 2.0 * oc.dot(&ray.direction);
                let c = oc.magnitude_squared() - radius_squared;
                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 {
                    return None;
                }

                let discriminant_sqrt = discriminant.sqrt();
                let inv_2_a = 1.0 / (2.0 * a);
                let mut distance = (-b - discriminant_sqrt) * inv_2_a;
                if let Some(location) = ray.update_max_distance(distance) {
                    return Some(ShapeIntersection {
                        location,
                        normal: (location - *origin) * *inv_radius,
                    });
                }

                distance = (-b + discriminant_sqrt) * inv_2_a;
                if let Some(location) = ray.update_max_distance(distance) {
                    return Some(ShapeIntersection {
                        location,
                        normal: (location - *origin) * *inv_radius,
                    });
                }

                None
            }
            Shape::Triangle {
                v0,
                e1,
                e2,
                _n0: n0,
                _n01: n01,
                _n02: n02,
            } => {
                // Source: http://www.graphics.cornell.edu/pubs/1997/MT97.pdf
                let P = ray.direction.cross(e2);

                let denominator = P.dot(e1);
                if denominator > -EPSILON && denominator < EPSILON {
                    return None;
                }

                let T = ray.origin - *v0;
                let inv_denominator = 1.0 / denominator;
                let u = P.dot(&T) * inv_denominator;
                if u < 0.0 || u > 1.0 {
                    return None;
                }

                let Q = T.cross(e1);
                let v = Q.dot(&ray.direction) * inv_denominator;
                if v < 0.0 || u + v > 1.0 {
                    return None;
                }

                let distance = T.cross(e1).dot(e2) * inv_denominator;
                if let Some(location) = ray.update_max_distance(distance) {
                    Some(ShapeIntersection {
                        location,
                        normal: (*n0 + *n01 * u + *n02 * v).normalized(),
                    })
                } else {
                    None
                }
            }
        }
    }
    pub fn bounds(&self) -> Bounds {
        match self {
            Shape::Sphere { origin, radius, .. } => Bounds::new(
                Vector(
                    origin.x() - radius,
                    origin.y() - radius,
                    origin.z() - radius,
                ),
                Vector(
                    origin.x() + radius,
                    origin.y() + radius,
                    origin.z() + radius,
                ),
            ),
            Shape::Triangle { v0, e1, e2, .. } => {
                let v1 = *v0 + *e1;
                let v2 = *v0 + *e2;

                Bounds::new(
                    Vector(
                        v1.x().min(v2.x().min(v0.x())),
                        v1.y().min(v2.y().min(v0.y())),
                        v1.z().min(v2.z().min(v0.z())),
                    ),
                    Vector(
                        v1.x().max(v2.x().max(v0.x())),
                        v1.y().max(v2.y().max(v0.y())),
                        v1.z().max(v2.z().max(v0.z())),
                    ),
                )
            }
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
                Shape::new_sphere(Vector(0.0, 0.0, 0.0), 1.0,).bounds(),
                Bounds::new(Vector(-1.0, -1.0, -1.0), Vector(1.0, 1.0, 1.0),)
            );

            assert_eq!(
                Shape::new_sphere(Vector(-2.0, 3.0, 0.0), 1.0,).bounds(),
                Bounds::new(Vector(-3.0, 2.0, -1.0), Vector(-1.0, 4.0, 1.0),)
            );
        }
    }

    mod triangle {
        use approx::assert_abs_diff_eq;
        use rand::{thread_rng, Rng};

        use super::*;

        // Triangle in XY plane
        fn triangle() -> Shape {
            Shape::new_triangle(
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
            match triangle() {
                Shape::Triangle { v0, e1, e2, .. } => {
                    for point in [v0, v0 + e1, v0 + e2] {
                        let ray = &mut Ray::new(Vector(point.x(), point.y(), -2.0), Vector::Z);
                        let intersection = t.intersect(ray).unwrap();
                        assert_eq!(ray.max_distance, 2.0);
                        assert_eq!(intersection.normal, Vector(0.0, 0.0, -1.0));
                    }
                }
                _ => unreachable!(),
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

            match triangle() {
                Shape::Triangle { v0, e1, e2, .. } => {
                    let mut rng = thread_rng();
                    let u = rng.gen_range(0.0..1.0);
                    let v = rng.gen_range(0.0..1.0);
                    let target = v0 + e1 * u + e2 * v;

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
                _ => unreachable!(),
            }
        }
    }
}
