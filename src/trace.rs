use approx::assert_abs_diff_eq;

use crate::{color::Color, constants::EPSILON, intersection::Intersection, ray::Ray, Scene};

fn get_nearest_intersection<'a>(ray: &Ray, scene: &'a Scene) -> Option<Intersection<'a>> {
    let mut nearest_intersection: Option<Intersection> = None;
    for shape in scene.shapes.iter() {
        if let Some(intersection) = shape.intersect(ray) {
            assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
            if intersection.distance > EPSILON
                && (nearest_intersection.is_none()
                    || intersection.distance < nearest_intersection.as_ref().unwrap().distance)
            {
                nearest_intersection = Some(intersection);
            }
        }
    }
    nearest_intersection
}

#[allow(non_snake_case)]
pub fn trace(ray: &Ray, scene: &Scene, depth: u32) -> Color {
    let depth = depth + 1;

    if let Some(intersection) = get_nearest_intersection(&ray, &scene) {
        let material = intersection.shape.material();

        let wo = ray.direction;
        let (wi, f, Le) = material.sample(&wo, &intersection.normal);

        let ray = Ray::new(intersection.location, wi);
        let Li = if depth <= scene.max_depth && !f.is_black() {
            trace(&ray, scene, depth - 1)
        } else {
            Color::BLACK
        };

        Le + Li * f
    } else {
        Color::BLACK
    }
}
