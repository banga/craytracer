use approx::assert_abs_diff_eq;
use rand::Rng;

use crate::{color::Color, constants::EPSILON, pdf::Pdf, ray::Ray, scene::Scene};

#[allow(non_snake_case)]
pub fn trace(mut ray: Ray, scene: &Scene) -> Color {
    let mut L = Color::WHITE;
    let mut rng = rand::thread_rng();

    loop {
        if L.is_black() {
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

        // Found a "light"
        if !surface_sample.Le.is_black() {
            assert!(
                surface_sample.f.is_black(),
                "Emissive surfaces should not have a BRDF"
            );
            return L * surface_sample.Le;
        }

        if surface_sample.f.is_black() {
            break;
        }

        let cos_theta = surface_sample.w_i.dot(&intersection.normal).abs();
        L = L * surface_sample.f * cos_theta;
        if let Pdf::NonDelta(pdf) = surface_sample.pdf {
            L = L / pdf;
        }

        // Very naive Russian Roulette
        let q: f64 = 0.05;
        if rng.gen_range(0.0..=1.0) < q {
            break;
        }
        L = L / (1.0 - q);

        ray = Ray::new(intersection.location, surface_sample.w_i);
    }

    Color::BLACK
}
