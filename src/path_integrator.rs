use crate::{
    bxdf::SurfaceSample,
    color::Color,
    geometry::traits::DotProduct,
    intersection::PrimitiveIntersection,
    light::LightSample,
    pdf::Pdf,
    ray::Ray,
    sampling::{
        samplers::{Sample1d, Sample2d, Sampler},
        sampling_fns::power_heuristic,
    },
    scene::Scene,
};

// This struct is meant to ensure that all path segments consume samples in the
// same order, so that for every pixel, they use the same dimension in the
// sampler, which is how the sampler ensures samples are well distributed.
struct PathSegmentSamples {
    material: (Sample1d, Sample2d),
    light_index: Sample1d,
    light: (Sample1d, Sample2d),
    russian_roulette: Sample1d,
}
impl PathSegmentSamples {
    fn from<S>(sampler: &mut S) -> PathSegmentSamples
    where
        S: Sampler,
    {
        PathSegmentSamples {
            material: (sampler.sample_1d(), sampler.sample_2d()),
            light_index: sampler.sample_1d(),
            light: (sampler.sample_1d(), sampler.sample_2d()),
            russian_roulette: sampler.sample_1d(),
        }
    }
}

/// Estimates light arriving along the given ray direction in the given scene
#[allow(non_snake_case)]
pub fn estimate_Li<S>(sampler: &mut S, mut ray: Ray, scene: &Scene) -> Color
where
    S: Sampler,
{
    let mut L = Color::BLACK;
    let mut beta = Color::WHITE;
    let mut bounces = 0;
    // This is initially set to true as a convenience for camera rays, since the
    // special cases below apply to them too
    let mut is_specular_bounce = true;
    let mut prev_bsdf_pdf = 0.0;
    let mut prev_intersection: Option<PrimitiveIntersection> = None;

    while bounces < scene.max_depth && !beta.is_black() {
        // Both `w_o` and `w_i` should be coming out of the surface
        let w_o = -ray.direction;

        let intersection = match scene.intersect(&mut ray) {
            Some(intersection) => intersection,
            None => {
                // If the path escapes the scene, account for infinite lights
                // which would not have been sampled if the previous direction
                // was sampled from a specular BRDF
                if is_specular_bounce {
                    for light in &scene.lights {
                        L += beta * light.Le(&w_o);
                    }
                } else {
                    // If the bounce was not specular, we can do MIS using the
                    // pdf for the bsdf that sampled this direction
                    let prev_intersection = prev_intersection.unwrap();
                    for (light_idx, light) in scene.lights.iter().enumerate() {
                        let Le = light.Le(&w_o);
                        if !Le.is_black() {
                            let light_pdf = match light.pdf_Li(&prev_intersection, &w_o) {
                                Pdf::NonDelta(pdf) => pdf,
                                Pdf::Delta => {
                                    unreachable!(
                                        "Emissive light {:?} should not be a delta light",
                                        light
                                    )
                                }
                            } * scene.light_sampler.pdf(light_idx);
                            let weight = power_heuristic(1, light_pdf, 1, prev_bsdf_pdf);
                            L += beta * Le * weight;
                        }
                    }
                }
                break;
            }
        };
        let PrimitiveIntersection {
            normal,
            location,
            material,
            uv,
            ..
        } = intersection;

        let path_samples = PathSegmentSamples::from(sampler);

        // If we hit an emissive surface (i.e. area light) and the direction was
        // generated via a specular bounce, then when we sampled the light in
        // the previous iteration, we would not have included its contribution,
        // so include it here
        let Le = intersection.Le(&w_o);
        if !Le.is_black() {
            if is_specular_bounce {
                L += beta * Le;
            } else {
                let light = intersection
                    .primitive
                    .get_area_light()
                    .expect("Expected area light for emissive interaction");
                // TODO: Avoid this linear search
                let light_idx = scene.lights.iter().position(|l| l == light).unwrap();
                let light_pdf = match light.pdf_Li(&intersection, &w_o) {
                    Pdf::NonDelta(pdf) => pdf,
                    Pdf::Delta => {
                        unreachable!("Emissive light {:?} should not be a delta light", light)
                    }
                } * scene.light_sampler.pdf(light_idx);
                let weight = power_heuristic(1, light_pdf, 1, prev_bsdf_pdf);
                L += beta * Le * weight;
            }
        }

        // Sample a light and add contribution
        {
            let (light_index, light_sampler_pdf) =
                scene.light_sampler.sample(path_samples.light_index);
            let light = &scene.lights[light_index];

            let LightSample {
                Li,
                w_i,
                pdf: light_pdf,
                shadow_ray,
            } = light.sample_Li(path_samples.light, &intersection);

            if !scene.intersects(&shadow_ray) {
                let f = material.f(&w_o, &w_i, &normal, &uv);
                let cos_theta = w_i.dot(&normal).abs();
                match light_pdf {
                    Pdf::NonDelta(light_pdf) => {
                        if light_pdf > 0.0 {
                            // If it's a non delta light, we can do MIS with the
                            // bsdf's pdf for the sampled direction
                            let light_pdf = light_pdf * light_sampler_pdf;
                            let bsdf_pdf = match material.pdf(&w_o, &w_i, &normal) {
                                Pdf::NonDelta(pdf) => pdf,
                                Pdf::Delta => 0.0,
                            };
                            let weight = power_heuristic(1, light_pdf, 1, bsdf_pdf);
                            L += beta * Li * f * cos_theta * weight / light_pdf;
                        }
                    }
                    Pdf::Delta => {
                        let light_pdf = light_sampler_pdf;
                        L += beta * Li * f * cos_theta / light_pdf;
                    }
                };
            }
        }

        // Sample the BRDF for the next direction
        {
            let SurfaceSample {
                w_i,
                f,
                pdf: bsdf_pdf,
                is_specular,
            } = match material.sample(path_samples.material, &w_o, &normal, &uv) {
                Some(surface_sample) => surface_sample,
                None => break,
            };
            if f.is_black() {
                break;
            }
            let cos_theta = w_i.dot(&normal).abs();
            let bsdf_pdf = match bsdf_pdf {
                Pdf::NonDelta(pdf) => pdf,
                Pdf::Delta => 1.0,
            };
            if bsdf_pdf == 0.0 {
                break;
            }

            beta = beta * f * cos_theta / bsdf_pdf;

            ray = Ray::new(location, w_i);
            is_specular_bounce = is_specular;
            prev_bsdf_pdf = bsdf_pdf;
            prev_intersection = Some(intersection);
        }

        if bounces > 0 {
            let max_beta_component = beta.r.max(beta.g.max(beta.b));
            if max_beta_component < 1.0 {
                let q = 1.0 - max_beta_component;
                if path_samples.russian_roulette.take() < q {
                    break;
                }
                beta = beta / (1.0 - q);
            }
        }

        assert!(L.is_finite());
        assert!(beta.is_finite());

        bounces += 1;
    }

    L
}
