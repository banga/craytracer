use approx::assert_abs_diff_eq;
use rand::Rng;

use crate::{
    color::Color, constants::EPSILON, intersection::PrimitiveIntersection, pdf::Pdf, ray::Ray,
    scene::Scene,
};

#[allow(non_snake_case)]
fn estimate_direct(intersection: &PrimitiveIntersection, scene: &Scene) -> Color {
    let mut rng = rand::thread_rng();

    if scene.lights.len() == 0 {
        return Color::BLACK;
    }

    let light = &scene.lights[rng.gen_range(0..scene.lights.len())];
    let mut light_sample = light.sample(&intersection.location);

    let shadow_intersection = scene.intersect(&mut light_sample.shadow_ray);
    if shadow_intersection.is_some() {
        return Color::BLACK;
    }

    // TODO: multiple importance sampling

    let cos_theta = light_sample.w_i.dot(&intersection.normal).abs();
    let mut Li = light_sample.Li * cos_theta;
    if let Pdf::NonDelta(pdf) = light_sample.pdf {
        Li = Li / pdf;
    }
    Li
}

/// As described in
/// https://pbr-book.org/3ed-2018/Light_Transport_I_Surface_Reflection/Path_Tracing,
/// this estimates the radiance arriving at the camera from a given ray by
/// constructing paths starting from the camera and ending at a light source and
/// summing the radiance along each path.
#[allow(non_snake_case)]
pub fn path_trace(mut ray: Ray, scene: &Scene) -> Color {
    let mut L = Color::BLACK;
    let mut beta = Color::WHITE;
    let mut rng = rand::thread_rng();
    let mut bounces = 0;

    loop {
        if bounces >= scene.max_depth {
            break;
        }

        if beta.is_black() {
            break;
        }

        let intersection = match scene.intersect(&mut ray) {
            Some(intersection) => intersection,
            None => break,
        };

        assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
        assert!(intersection.distance >= 0.0);

        let w_o = ray.direction;
        let surface_sample = match intersection.material.sample(&w_o, &intersection.normal) {
            Some(surface_sample) => surface_sample,
            None => break,
        };

        if surface_sample.f.is_black() {
            break;
        }

        let cos_theta = surface_sample.w_i.dot(&intersection.normal).abs();
        beta = beta * surface_sample.f * cos_theta;
        if let Pdf::NonDelta(pdf) = surface_sample.pdf {
            beta = beta / pdf;
        }

        // Estimate the contribution from a path that ends here. We will reuse
        // the path without the terminator in the loop.
        L += beta * estimate_direct(&intersection, scene);

        // Very naive Russian Roulette
        let q: f64 = 0.05_f64.max(1.0 - (beta.r + beta.g + beta.b) * 0.3);
        if bounces > 3 && rng.gen_range(0.0..1.0) < q {
            break;
        }
        beta = beta / (1.0 - q);

        ray = Ray::new(intersection.location, surface_sample.w_i);

        bounces += 1;
    }

    L
}
