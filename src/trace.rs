use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{traits::DotProduct, vector::Vector},
    intersection::PrimitiveIntersection,
    light::Light,
    pdf::Pdf,
    ray::Ray,
    sampling::power_heuristic,
    scene::Scene,
};
use approx::assert_abs_diff_eq;
use rand::Rng;

/// Estimate the radiance leaving the given point in the direction w_o from the
/// given light source.
#[allow(non_snake_case)]
fn estimate_direct<R>(
    rng: &mut R,
    intersection: &PrimitiveIntersection,
    w_o: &Vector,
    light: &Light,
    scene: &Scene,
) -> Color
where
    R: Rng,
{
    let mut Ld = Color::BLACK;

    // Sample the light source
    let mut light_sample = light.sample(rng, &intersection.location);
    let shadow_intersection = scene.intersect(&mut light_sample.shadow_ray);
    if shadow_intersection.is_none() {
        let cos_theta_i = light_sample.w_i.dot(&intersection.normal).abs();
        let mut Li = intersection
            .material
            .f(w_o, &light_sample.w_i, &intersection.normal)
            * light_sample.Li
            * cos_theta_i;
        if !Li.is_black() {
            let pdf_f = intersection
                .material
                .pdf(w_o, &light_sample.w_i, &intersection.normal);
            let pdf_f = match pdf_f {
                Pdf::Delta => 0.0,
                Pdf::NonDelta(pdf_f) => pdf_f,
            };
            let mut weight = 1.0;
            if let Pdf::NonDelta(pdf_light) = light_sample.pdf {
                weight = power_heuristic(1, pdf_light, 1, pdf_f);
                Li = Li / pdf_light;
            }
            Ld += Li * weight;
        }
    }

    // Sample the BRDF
    {
        if let Some(material_sample) = intersection.material.sample(rng, w_o, &intersection.normal)
        {
            let ray = &mut Ray::new(intersection.location, material_sample.w_i);
            if scene.intersect(ray).is_some() {
                // TODO: Implement area lights
                return Ld;
            }

            // If the light's direction is delta distributed, there's no chance
            // the BRDF would sample it, so we only add the contribution if it's
            // a non-delta light
            if let Pdf::NonDelta(pdf_light) = light.pdf(&material_sample.w_i) {
                let cos_theta_i = material_sample.w_i.dot(&intersection.normal).abs();
                let mut Li = light.Le(&material_sample.w_i) * material_sample.f * cos_theta_i;
                let mut weight = 1.0;
                if let Pdf::NonDelta(pdf_f) = material_sample.pdf {
                    weight = power_heuristic(1, pdf_f, 1, pdf_light);
                    Li = Li / pdf_f;
                }
                Ld += Li * weight;
            }
        }
    }

    Ld
}

/// As described in
/// https://pbr-book.org/3ed-2018/Light_Transport_I_Surface_Reflection/Path_Tracing,
/// this estimates the radiance arriving at the camera from a given ray by
/// constructing paths starting from the camera and ending at a light source and
/// summing the radiance along each path.
#[allow(non_snake_case)]
pub fn path_trace<R>(rng: &mut R, mut ray: Ray, scene: &Scene) -> Color
where
    R: Rng,
{
    assert!(scene.lights.len() > 0, "No lights in the scene.");

    let mut L = Color::BLACK;
    let mut beta = Color::WHITE;
    let mut bounces = 0;
    let mut is_specular_bounce = false;

    loop {
        if bounces >= scene.max_depth {
            break;
        }

        if beta.is_black() {
            break;
        }

        let intersection = match scene.intersect(&mut ray) {
            Some(intersection) => intersection,
            None => {
                if is_specular_bounce || bounces == 0 {
                    for light in &scene.lights {
                        L += beta * light.Le(&ray.direction);
                    }
                }
                break;
            }
        };

        assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
        assert!(intersection.distance >= 0.0);

        let w_o = ray.direction;
        let surface_sample = match intersection
            .material
            .sample(rng, &w_o, &intersection.normal)
        {
            Some(surface_sample) => surface_sample,
            None => break,
        };

        if surface_sample.f.is_black() {
            break;
        }

        is_specular_bounce = surface_sample.is_specular;

        // Estimate the contribution from a path that ends here. We will reuse
        // the path without the terminator in the loop.
        let light_pdf = 1.0 / scene.lights.len() as f64;
        let light = &scene.lights[rng.gen_range(0..scene.lights.len())];
        L += beta * estimate_direct(rng, &intersection, &ray.direction, &light, scene) / light_pdf;

        let cos_theta = surface_sample.w_i.dot(&intersection.normal).abs();
        beta = beta * surface_sample.f * cos_theta;
        if let Pdf::NonDelta(pdf) = surface_sample.pdf {
            beta = beta / pdf;
        }

        // Very naive Russian Roulette
        if bounces > 3 {
            let q: f64 = 0.05_f64.max(1.0 - (beta.r + beta.g + beta.b) * 0.3);
            if rng.gen_range(0.0..1.0) < q {
                break;
            }
            beta = beta / (1.0 - q);
        }

        ray = Ray::new(intersection.location, surface_sample.w_i);

        bounces += 1;
    }

    L
}
