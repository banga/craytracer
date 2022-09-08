use approx::assert_abs_diff_eq;

use crate::{color::Color, constants::EPSILON, pdf::Pdf, ray::Ray, Scene};

#[allow(non_snake_case)]
pub fn trace(ray: &mut Ray, scene: &Scene, depth: u32) -> Color {
    let depth = depth + 1;

    if let Some(intersection) = scene.bvh.intersect(ray) {
        assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
        assert!(intersection.distance >= 0.0);

        let w_o = ray.direction;
        if let Some(surface_sample) = intersection.material.sample(&w_o, &intersection.normal) {
            let mut ray = Ray::new(intersection.location, surface_sample.w_i);
            if depth <= scene.max_depth && !surface_sample.f.is_black() {
                let Li = trace(&mut ray, scene, depth);
                let cos_theta = surface_sample.w_i.dot(&intersection.normal).abs();
                let mut Lo = Li * surface_sample.f * cos_theta;
                if let Pdf::NonDelta(pdf) = surface_sample.pdf {
                    Lo = Lo / pdf;
                }
                surface_sample.Le + Lo
            } else {
                surface_sample.Le
            }
        } else {
            Color::BLACK
        }
    } else {
        Color::BLACK
    }
}
