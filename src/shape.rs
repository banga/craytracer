use std::{
    f64::consts::{FRAC_1_PI, PI},
    sync::Arc,
};

use crate::{
    bounds::Bounds,
    constants::EPSILON,
    geometry::{normal::Normal, point::Point, traits::DotProduct, vector::Vector, O},
    intersection::{PrimitiveIntersection, ShapeIntersection},
    pdf::Pdf,
    ray::Ray,
    sampling::{
        samplers::Sample2d,
        sampling_fns::{sample_disk, sample_sphere, sample_triangle},
    },
    transformation::{Transformable, Transformation},
    v,
};

// TODO: Avoid this PartialEq, currently used by path_integrator to map an area
// light to index in the lights array
#[derive(Debug, PartialEq)]
pub enum Shape {
    Sphere {
        object_to_world: Arc<Transformation>,
        world_to_object: Arc<Transformation>,
        radius: f64,
    },
    Triangle {
        v0: Point,
        e1: Vector,
        e2: Vector,
        n0: Vector,
        n01: Vector,
        n02: Vector,
        uv0: (f64, f64),
        uv01: (f64, f64),
        uv02: (f64, f64),
    },
    Disk {
        object_to_world: Arc<Transformation>,
        world_to_object: Arc<Transformation>,
        radius: f64,
        inner_radius: f64,
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
            object_to_world: Arc::new(Transformation::translate(
                origin.x(),
                origin.y(),
                origin.z(),
            )),
            world_to_object: Arc::new(Transformation::translate(
                -origin.x(),
                -origin.y(),
                -origin.z(),
            )),
        }
    }
    pub fn new_triangle(v0: Point, v1: Point, v2: Point) -> Option<Shape> {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        // Assuming that vertices are in clockwise order, calculate the normal
        // in a left handed co-ordinate system:
        let mut n0 = e2.cross(&e1);
        let magnitude = n0.magnitude();
        if magnitude == 0.0 {
            // Degenerate triangle
            return None;
        }
        n0 = n0 / magnitude;

        Some(Shape::Triangle {
            v0,
            e1,
            e2,
            n0,
            n01: v!(0, 0, 0),
            n02: v!(0, 0, 0),
            uv0: (0.0, 0.0),
            uv01: (1.0, 0.0),
            uv02: (1.0, 1.0),
        })
    }
    pub fn new_triangle_with_normals_and_texture_coordinates(
        v0: Point,
        v1: Point,
        v2: Point,
        n0: Vector,
        n1: Vector,
        n2: Vector,
        uv0: (f64, f64),
        uv1: (f64, f64),
        uv2: (f64, f64),
    ) -> Option<Shape> {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        if e2.cross(&e1).magnitude_squared() == 0.0
            || n0.magnitude_squared() == 0.0
            || n1.magnitude_squared() == 0.0
            || n2.magnitude_squared() == 0.0
        {
            return None;
        }

        let uv01 = (uv1.0 - uv0.0, uv1.1 - uv0.1);
        let uv02 = (uv2.0 - uv0.0, uv2.1 - uv0.1);

        Some(Shape::Triangle {
            v0,
            e1,
            e2,
            n0,
            n01: n1 - n0,
            n02: n2 - n0,
            uv0,
            uv01,
            uv02,
        })
    }
    pub fn new_disk(
        origin: Point,
        rotate_x: f64,
        rotate_y: f64,
        radius: f64,
        inner_radius: f64,
    ) -> Shape {
        let object_to_world = Arc::new(
            Transformation::translate(origin.x(), origin.y(), origin.z())
                * Transformation::rotate_x(rotate_x.to_radians())
                * Transformation::rotate_y(rotate_y.to_radians()),
        );
        let world_to_object = Arc::new(object_to_world.inverse());

        Shape::Disk {
            radius,
            inner_radius,
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
                radius,
            } => {
                let mut obj_ray = world_to_object.transform(ray);

                let oc = Vector(obj_ray.origin.x(), obj_ray.origin.y(), obj_ray.origin.z());
                let a = obj_ray.direction.magnitude_squared();
                let b = 2.0 * oc.dot(&obj_ray.direction);
                let c = oc.magnitude_squared() - radius.powf(2.0);
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
                    let mut phi = location.y().atan2(location.x());
                    if phi < 0.0 {
                        phi += PI * 2.0;
                    }
                    let u = phi / (PI * 2.0);
                    let theta = (location.z() / radius).acos();
                    let v = theta * FRAC_1_PI;
                    return Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal(location.x(), location.y(), location.z()) / *radius,
                        uv: (u, v),
                    }));
                }

                distance = (-b + discriminant_sqrt) * inv_2_a;
                if obj_ray.update_max_distance(distance) {
                    let location = obj_ray.at(distance);
                    ray.update_max_distance(distance);
                    let mut phi = location.y().atan2(location.x());
                    if phi < 0.0 {
                        phi += PI * 2.0;
                    }
                    let u = phi / (PI * 2.0);
                    let theta = (location.z() / radius).acos();
                    let v = theta * FRAC_1_PI;
                    return Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal(location.x(), location.y(), location.z()) / *radius,
                        uv: (u, v),
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
                uv0,
                uv01,
                uv02,
            } => {
                // Source: http://www.graphics.cornell.edu/pubs/1997/MT97.pdf
                let P = ray.direction.cross(e2);

                let denominator = P.dot(e1);
                if denominator > -EPSILON && denominator < EPSILON {
                    return None;
                }

                let T = ray.origin - *v0;
                let u = P.dot(&T) / denominator;
                if u < 0.0 || u > 1.0 {
                    return None;
                }

                let Q = T.cross(e1);
                let v = Q.dot(&ray.direction) / denominator;
                if v < 0.0 || u + v > 1.0 {
                    return None;
                }

                let distance = T.cross(e1).dot(e2) / denominator;
                if ray.update_max_distance(distance) {
                    let location = ray.at(distance);

                    Some(ShapeIntersection {
                        location,
                        normal: (*n0 + *n01 * u + *n02 * v).normalized().into(),
                        uv: (
                            uv0.0 + uv01.0 * u + uv02.0 * v,
                            uv0.1 + uv01.1 * u + uv02.1 * v,
                        ),
                    })
                } else {
                    None
                }
            }
            Shape::Disk {
                object_to_world,
                world_to_object,
                radius,
                inner_radius,
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
                if distance_squared < inner_radius.powf(2.0) || distance_squared > radius.powf(2.0)
                {
                    return None;
                }

                let mut theta = location.y().atan2(location.x());
                if theta < 0.0 {
                    theta += PI * 2.0;
                }
                let u = theta / (PI * 2.0);
                let v = distance_squared.sqrt() / radius;

                if ray.update_max_distance(t) {
                    Some(object_to_world.transform(&ShapeIntersection {
                        location,
                        normal: Normal::Z,
                        uv: (u, v),
                    }))
                } else {
                    None
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn intersects(&self, ray: &Ray) -> bool {
        match self {
            Shape::Sphere {
                world_to_object,
                radius,
                ..
            } => {
                let obj_ray = world_to_object.transform(ray);

                let oc = Vector(obj_ray.origin.x(), obj_ray.origin.y(), obj_ray.origin.z());
                let a = obj_ray.direction.magnitude_squared();
                let b = 2.0 * oc.dot(&obj_ray.direction);
                let c = oc.magnitude_squared() - radius.powf(2.0);
                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 {
                    return false;
                }

                let discriminant_sqrt = discriminant.sqrt();
                let inv_2_a = 1.0 / (2.0 * a);
                let mut distance = (-b - discriminant_sqrt) * inv_2_a;
                if obj_ray.contains_distance(distance) {
                    return true;
                }

                distance = (-b + discriminant_sqrt) * inv_2_a;
                obj_ray.contains_distance(distance)
            }
            Shape::Triangle { v0, e1, e2, .. } => {
                // Source: http://www.graphics.cornell.edu/pubs/1997/MT97.pdf
                let P = ray.direction.cross(e2);

                let denominator = P.dot(e1);
                if denominator > -EPSILON && denominator < EPSILON {
                    return false;
                }

                let T = ray.origin - *v0;
                let u = P.dot(&T) / denominator;
                if u < 0.0 || u > 1.0 {
                    return false;
                }

                let Q = T.cross(e1);
                let v = Q.dot(&ray.direction) / denominator;
                if v < 0.0 || u + v > 1.0 {
                    return false;
                }

                let distance = T.cross(e1).dot(e2) / denominator;
                ray.contains_distance(distance)
            }
            Shape::Disk {
                world_to_object,
                radius,
                inner_radius,
                ..
            } => {
                let obj_ray = world_to_object.transform(ray);
                if obj_ray.direction.z() == 0.0 {
                    return false;
                }
                // The ray will intersect the plane at z = 0.
                // So, o.z + d.z * t = 0
                let t = -obj_ray.origin.z() / obj_ray.direction.z();
                if !obj_ray.contains_distance(t) {
                    return false;
                }

                // If the distance of the intersection from (0, 0, 0) is in
                // [inner_radius, radius], then we have found an intersection
                let location = Point(
                    obj_ray.origin.x() + obj_ray.direction.x() * t,
                    obj_ray.origin.y() + obj_ray.direction.y() * t,
                    0.0,
                );
                let distance_squared = location.x().powf(2.0) + location.y().powf(2.0);
                if distance_squared < inner_radius.powf(2.0) || distance_squared > radius.powf(2.0)
                {
                    return false;
                }

                ray.contains_distance(t)
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
    /// So far, these are only used for area lights.

    /// Samples a point uniformly on the surface of the shape
    pub fn sample(&self, point_sample: Sample2d) -> Point {
        match &self {
            Shape::Sphere {
                object_to_world,
                radius,
                ..
            } => {
                let point = O + sample_sphere(point_sample) * *radius;
                object_to_world.transform(&point)
            }
            Shape::Triangle { v0, e1, e2, .. } => {
                let (b1, b2) = sample_triangle(point_sample);
                *v0 + *e1 * b1 + *e2 * b2
            }
            Shape::Disk {
                object_to_world,
                radius,
                ..
            } => {
                // Note: this does not account for inner_radius
                let (x, y) = sample_disk(point_sample);
                let point = Point(x * radius, y * radius, 0.0);
                object_to_world.transform(&point)
            }
        }
    }

    pub fn sample_from(
        &self,
        point_sample: Sample2d,
        intersection: &PrimitiveIntersection,
    ) -> (Point, Vector, Pdf) {
        // TODO: We should use a better method than sampling the surface of the
        // shape uniformly. It's currently possible that we will return a point
        // that is not actually visible from the intersection.
        let point = self.sample(point_sample);
        let w_i = (point - intersection.location).normalized();
        let pdf = self.pdf_from(intersection, &w_i);
        (point, w_i, pdf)
    }

    /// Pdf for sampling in the given direction on this shape from the given intersection
    pub fn pdf_from(&self, intersection: &PrimitiveIntersection, w_i: &Vector) -> Pdf {
        let mut ray = Ray::new(intersection.location, *w_i);
        match self.intersect(&mut ray) {
            Some(shape_intersection) => {
                let distance_squared =
                    (shape_intersection.location - intersection.location).magnitude_squared();
                let cos_theta = w_i.dot(&intersection.normal).abs();
                let pdf = distance_squared / (cos_theta * self.area());
                Pdf::NonDelta(pdf)
            }
            // We should ideally never sample a direction that does not hit this
            // shape, so the pdf should be 0.0. This not currently true, since
            // we sample the surface uniformly (see above)
            None => Pdf::NonDelta(0.0),
        }
    }

    pub fn area(&self) -> f64 {
        match &self {
            Shape::Sphere { radius, .. } => PI * radius.powf(2.0),
            Shape::Triangle { e1, e2, .. } => e1.cross(e2).magnitude() / 2.0,
            Shape::Disk {
                radius,
                inner_radius,
                ..
            } => PI * (radius.powf(2.0) - inner_radius.powf(2.0)),
        }
    }
}
