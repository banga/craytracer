use approx::assert_abs_diff_eq;

use crate::{color::Color, constants::EPSILON, ray::Ray, Scene};

#[allow(non_snake_case)]
pub fn trace(ray: &mut Ray, scene: &Scene, depth: u32) -> Color {
    let depth = depth + 1;

    if let Some(intersection) = scene.bvh.intersect(ray) {
        assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
        assert!(intersection.distance >= 0.0);

        let wo = ray.direction;
        let (wi, f, Le) = intersection.material.sample(&wo, &intersection.normal);

        let mut ray = Ray::new(intersection.location, wi);
        let Li = if depth <= scene.max_depth && !f.is_black() {
            trace(&mut ray, scene, depth)
        } else {
            Color::BLACK
        };

        Le + Li * f
    } else {
        Color::BLACK
    }
}
