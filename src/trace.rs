use approx::assert_abs_diff_eq;

use crate::{color::Color, constants::EPSILON, pdf::Pdf, ray::Ray, scene::Scene};

#[allow(non_snake_case)]
pub fn trace(mut ray: Ray, scene: &Scene) -> Color {
    let mut L = Color::WHITE;
    let mut depth = 0;

    while depth < scene.max_depth && !L.is_black() {
        match scene.intersect(&mut ray) {
            Some(intersection) => {
                assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
                assert!(intersection.distance >= 0.0);

                let w_o = ray.direction;
                match intersection.material.sample(&w_o, &intersection.normal) {
                    Some(surface_sample) => {
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

                        ray = Ray::new(intersection.location, surface_sample.w_i);
                    }
                    None => break,
                }
            }
            None => break,
        }

        depth += 1;
    }

    Color::BLACK
}
