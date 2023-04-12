use crate::{
    bounds::Bounds,
    constants::EPSILON,
    geometry::{point::Point, vector::Vector},
    intersection::ShapeIntersection,
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub enum Shape {
    Sphere {
        origin: Point,
        radius: f64,
        radius_squared: f64,
        inv_radius: f64,
    },
    Triangle {
        v0: Point,
        e1: Vector,
        e2: Vector,
        n0: Vector,
        n01: Vector,
        n02: Vector,
    },
}

impl Shape {
    pub fn new_sphere(origin: Point, radius: f64) -> Shape {
        Shape::Sphere {
            origin,
            radius,
            radius_squared: radius * radius,
            inv_radius: 1.0 / radius,
        }
    }
    pub fn new_triangle(v0: Point, v1: Point, v2: Point) -> Shape {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        // Assuming that vertices are in clockwise order, calculate the normal
        // in a left handed co-ordinate system:
        let n0 = e2.cross(&e1).normalized();

        Shape::Triangle {
            v0,
            e1,
            e2,
            n0,
            n01: Vector::NULL,
            n02: Vector::NULL,
        }
    }
    pub fn new_triangle_with_normals(
        v0: Point,
        v1: Point,
        v2: Point,
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
            n0,
            n01: n1 - n0,
            n02: n2 - n0,
        }
    }

    #[allow(non_snake_case)]
    // Should update the ray's max_distance if an intersection is found
    pub fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection> {
        match self {
            Shape::Sphere {
                origin,
                radius_squared,
                inv_radius,
                ..
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
                n0,
                n01,
                n02,
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
                Point(
                    origin.x() - radius,
                    origin.y() - radius,
                    origin.z() - radius,
                ),
                Point(
                    origin.x() + radius,
                    origin.y() + radius,
                    origin.z() + radius,
                ),
            ),
            Shape::Triangle { v0, e1, e2, .. } => {
                let v1 = *v0 + *e1;
                let v2 = *v0 + *e2;

                Bounds::new(
                    Point(
                        v1.x().min(v2.x().min(v0.x())),
                        v1.y().min(v2.y().min(v0.y())),
                        v1.z().min(v2.z().min(v0.z())),
                    ),
                    Point(
                        v1.x().max(v2.x().max(v0.x())),
                        v1.y().max(v2.y().max(v0.y())),
                        v1.z().max(v2.z().max(v0.z())),
                    ),
                )
            }
        }
    }
}
