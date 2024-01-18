use std::f64::consts::PI;

use rand::Rng;

use crate::{
    bounds::Bounds,
    constants::EPSILON,
    geometry::{normal::Normal, point::Point, traits::DotProduct, vector::Vector, O},
    intersection::ShapeIntersection,
    pdf::Pdf,
    ray::Ray,
    sampling::{sample_disk, sample_sphere},
    transformation::{Transformable, Transformation},
    v,
};

#[derive(Debug, PartialEq)]
pub enum Shape {
    Sphere {
        object_to_world: Transformation,
        world_to_object: Transformation,
        radius: f64,
        radius_squared: f64,
        inv_radius: f64,
        area: f64,
    },
    Triangle {
        v0: Point,
        e1: Vector,
        e2: Vector,
        n0: Vector,
        n01: Vector,
        n02: Vector,
    },
    Disk {
        object_to_world: Transformation,
        world_to_object: Transformation,
        radius: f64,
        radius_squared: f64,
        inner_radius_squared: f64,
        area: f64,
    },
}

pub struct ShapeSample {
    pub point: Point,
    pub w_i: Vector,
}

impl Shape {
    pub fn new_sphere(origin: Point, radius: f64) -> Shape {
        Shape::Sphere {
            radius,
            radius_squared: radius * radius,
            inv_radius: 1.0 / radius,
            area: radius * radius * 4.0 * PI,
            object_to_world: Transformation::translate(origin.x(), origin.y(), origin.z()),
            world_to_object: Transformation::translate(-origin.x(), -origin.y(), -origin.z()),
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
            n01: v!(0, 0, 0),
            n02: v!(0, 0, 0),
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
    pub fn new_disk(
        origin: Point,
        rotate_x: f64,
        rotate_y: f64,
        radius: f64,
        inner_radius: f64,
    ) -> Shape {
        let object_to_world = Transformation::translate(origin.x(), origin.y(), origin.z())
            * Transformation::rotate_x(rotate_x.to_radians())
            * Transformation::rotate_y(rotate_y.to_radians());
        let world_to_object = object_to_world.inverse();

        Shape::Disk {
            radius,
            radius_squared: radius.powf(2.0),
            inner_radius_squared: inner_radius.powf(2.0),
            area: PI * (radius.powf(2.0) - inner_radius.powf(2.0)),
            object_to_world,
            world_to_object,
        }
    }

    #[allow(non_snake_case)]
    // Should update the ray's max_distance if an intersection is found
    pub fn intersect(&self, ray: &mut Ray) -> Option<ShapeIntersection> {
        match self {
            Shape::Sphere {
                object_to_world,
                world_to_object,
                radius_squared,
                inv_radius,
                ..
            } => {
                let mut obj_ray = world_to_object.transform(ray);

                let oc = Vector(obj_ray.origin.x(), obj_ray.origin.y(), obj_ray.origin.z());
                let a = obj_ray.direction.magnitude_squared();
                let b = 2.0 * oc.dot(&obj_ray.direction);
                let c = oc.magnitude_squared() - *radius_squared;
                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 {
                    return None;
                }

                let discriminant_sqrt = discriminant.sqrt();
                let inv_2_a = 1.0 / (2.0 * a);
                let mut distance = (-b - discriminant_sqrt) * inv_2_a;
                if obj_ray.update_max_distance(distance) {
                    let location = obj_ray.at(distance);
                    ray.update_max_distance(distance);
                    return Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal(location.x(), location.y(), location.z()) * *inv_radius,
                    }));
                }

                distance = (-b + discriminant_sqrt) * inv_2_a;
                if obj_ray.update_max_distance(distance) {
                    let location = obj_ray.at(distance);
                    ray.update_max_distance(distance);
                    return Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal(location.x(), location.y(), location.z()) * *inv_radius,
                    }));
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
                if ray.update_max_distance(distance) {
                    let location = ray.at(distance);
                    Some(ShapeIntersection {
                        location,
                        normal: (*n0 + *n01 * u + *n02 * v).normalized().into(),
                    })
                } else {
                    None
                }
            }
            Shape::Disk {
                object_to_world,
                world_to_object,
                radius_squared,
                inner_radius_squared,
                ..
            } => {
                let obj_ray = world_to_object.transform(ray);
                if obj_ray.direction.z() == 0.0 {
                    return None;
                }
                // The ray will intersect the plane at z = 0.
                // So, o.z + d.z * t = 0
                let t = -obj_ray.origin.z() / obj_ray.direction.z();
                if !obj_ray.contains_distance(t) {
                    return None;
                }

                // If the distance of the intersection from (0, 0, 0) is in
                // [inner_radius, radius], then we have found an intersection
                let location = Point(
                    obj_ray.origin.x() + obj_ray.direction.x() * t,
                    obj_ray.origin.y() + obj_ray.direction.y() * t,
                    0.0,
                );
                let distance_squared = location.x().powf(2.0) + location.y().powf(2.0);
                if distance_squared < *inner_radius_squared || distance_squared > *radius_squared {
                    return None;
                }

                if ray.update_max_distance(t) {
                    Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal::Z,
                    }))
                } else {
                    None
                }
            }
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Shape::Sphere {
                object_to_world,
                radius,
                ..
            } => object_to_world.transform(&Bounds::new(
                Point(-radius, -radius, -radius),
                Point(*radius, *radius, *radius),
            )),
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
            Shape::Disk {
                radius,
                object_to_world,
                ..
            } => object_to_world.transform(&Bounds::new(
                Point(-*radius, -*radius, 0.0),
                Point(*radius, *radius, 0.0),
            )),
        }
    }

    /// The sampling methods below are described in
    /// https://www.pbr-book.org/3ed-2018/Light_Transport_I_Surface_Reflection/Sampling_Light_Sources#SamplingShapes
    /// So far, these are only used for area lights. They are not implemented
    /// for triangles yet.

    /// Samples a point on the surface of the shape
    pub fn sample<R>(&self, rng: &mut R) -> Point
    where
        R: Rng,
    {
        match &self {
            Shape::Sphere {
                object_to_world,
                radius,
                ..
            } => {
                let point = O + sample_sphere(rng) * *radius;
                object_to_world.transform(&point)
            }
            Shape::Triangle { .. } => todo!(),
            Shape::Disk {
                object_to_world,
                radius,
                ..
            } => {
                // Note: this does not account for inner_radius
                let [x, y] = sample_disk(rng);
                let point = Point(x * radius, y * radius, 0.0);
                object_to_world.transform(&point)
            }
        }
    }

    /// Pdf w.r.t. solid angle for sampling the given direction from the given
    /// point.
    pub fn pdf(&self, point: &Point, w_i: &Vector) -> Pdf {
        let mut ray = Ray::new(*point, *w_i);
        match self.intersect(&mut ray) {
            None => Pdf::Delta,
            Some(intersection) => {
                let cos_theta = intersection.normal.dot(w_i).abs();
                if cos_theta == 0.0 {
                    return Pdf::Delta;
                }
                let pdf_area = 1.0 / self.area();
                let distance_squared = (*point - intersection.location).magnitude_squared();
                Pdf::NonDelta(pdf_area * distance_squared / cos_theta)
            }
        }
    }

    pub fn area(&self) -> f64 {
        match &self {
            Shape::Sphere { area, .. } => *area,
            Shape::Triangle { .. } => todo!(),
            Shape::Disk { area, .. } => *area,
        }
    }
}
