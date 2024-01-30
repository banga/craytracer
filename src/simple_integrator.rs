use crate::{
    bxdf::SurfaceSample,
    color::Color,
    geometry::traits::DotProduct,
    intersection::PrimitiveIntersection,
    light::LightSample,
    pdf::Pdf,
    ray::Ray,
    sampling::samplers::{Sample1d, Sample2d, Sampler},
    scene::Scene,
};

// This struct is meant to ensure that all path segments consume samples in the
// same order, so that for every pixel, they use the same dimension in the
// sampler, which is how the sampler ensures samples are well distributed.
struct PathSegmentSamples {
    material: (Sample1d, Sample2d),
    light_index: Sample1d,
    light: (Sample1d, Sample2d),
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
        if is_specular_bounce {
            L += beta * intersection.Le(&w_o);
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

            let light_pdf = match light_pdf {
                Pdf::NonDelta(pdf) => pdf,
                Pdf::Delta => 1.0,
            };

            if light_pdf > 0.0 && !scene.intersects(&shadow_ray) {
                let f = material.f(&w_o, &w_i, &normal, &uv);
                let cos_theta = w_i.dot(&normal).abs();
                L += beta * Li * f * cos_theta / light_sampler_pdf / light_pdf;
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
        }

        assert!(L.is_finite());
        assert!(beta.is_finite());

        bounces += 1;
    }

    L
}
