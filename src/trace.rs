use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{traits::DotProduct, vector::Vector},
    intersection::PrimitiveIntersection,
    light::Light,
    pdf::Pdf,
    ray::Ray,
    sampling::{
        samplers::{Sample1d, Sample2d, Sampler},
        sampling_fns::power_heuristic,
    },
    scene::Scene,
};
use approx::assert_abs_diff_eq;

#[allow(non_snake_case)]
fn sample_light(
    samples: (Sample1d, Sample2d),
    intersection: &PrimitiveIntersection<'_>,
    w_o: &Vector,
    light: &Light,
    scene: &Scene,
) -> Color {
    let light_sample = light.sample_Li(samples, &intersection);
    if let Pdf::NonDelta(pdf) = light_sample.pdf {
        if pdf == 0.0 {
            return Color::BLACK;
        }
    }
    if light_sample.Li.is_black() {
        return Color::BLACK;
    }

    if scene.intersects(&light_sample.shadow_ray) {
        return Color::BLACK;
    }
    let f = intersection
        .material
        .f(w_o, &light_sample.w_i, &intersection.normal);
    if f.is_black() {
        return Color::BLACK;
    }

    let cos_theta_i = light_sample.w_i.dot(&intersection.normal).abs();
    let mut Li = f * light_sample.Li * cos_theta_i;
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
    Li * weight
}

#[allow(non_snake_case)]
fn sample_brdf(
    samples: (Sample1d, Sample2d),
    intersection: &PrimitiveIntersection<'_>,
    w_o: &Vector,
    light: &Light,
    scene: &Scene,
) -> Color {
    let material_sample = intersection
        .material
        .sample(samples, w_o, &intersection.normal);
    if material_sample.is_none() {
        return Color::BLACK;
    }
    let material_sample = material_sample.unwrap();

    let mut Li = Color::BLACK;
    let mut weight = 1.0;

    let ray = &mut Ray::new(intersection.location, material_sample.w_i);
    // If the sampled direction hits something in the scene, it can only
    // contribute if the thing it hits is the area light we are sampling
    if let Some(surface_intersection) = scene.intersect(ray) {
        if let Some(area_light) = surface_intersection.primitive.get_area_light() {
            if light == area_light.as_ref() {
                Li = light.L(&surface_intersection, &material_sample.w_i);
            }
        }
    } else if
    // If the light's direction is delta distributed, there's no chance the BRDF
    // would sample it, so we only add the contribution if it's a non-delta
    // light
    let Pdf::NonDelta(pdf_light) = light.pdf_Li(&intersection, &material_sample.w_i) {
        Li = light.Le(&material_sample.w_i);
        if let Pdf::NonDelta(pdf_f) = material_sample.pdf {
            weight = power_heuristic(1, pdf_f, 1, pdf_light);
            Li = Li / pdf_f;
        }
    }

    if Li.is_black() {
        return Color::BLACK;
    }

    let cos_theta_i = material_sample.w_i.dot(&intersection.normal).abs();
    Li * weight * material_sample.f * cos_theta_i
}

/// Estimate the radiance leaving the given point in the direction w_o from the
/// given light source.
#[allow(non_snake_case)]
fn estimate_direct(
    light_samples: (Sample1d, Sample2d),
    brdf_samples: (Sample1d, Sample2d),
    intersection: &PrimitiveIntersection,
    w_o: &Vector,
    light: &Light,
    scene: &Scene,
) -> Color {
    let mut Ld = Color::BLACK;

    Ld += sample_light(light_samples, intersection, w_o, light, scene);
    Ld += sample_brdf(brdf_samples, intersection, w_o, light, scene);

    Ld
}

// This struct is meant to ensure that all path segments consume samples in the
// same order, so that for every pixel, they use the same dimension in the
// sampler, which is how the sampler ensures samples are well distributed.
struct PathSegmentSamples {
    material: (Sample1d, Sample2d),
    light_index: Sample1d,
    light: (Sample1d, Sample2d),
    brdf: (Sample1d, Sample2d),
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
            brdf: (sampler.sample_1d(), sampler.sample_2d()),
            russian_roulette: sampler.sample_1d(),
        }
    }
}

/// As described in
/// https://pbr-book.org/3ed-2018/Light_Transport_I_Surface_Reflection/Path_Tracing,
/// this estimates the radiance arriving at the camera from a given ray by
/// constructing paths starting from the camera and ending at a light source and
/// summing the radiance along each path.
#[allow(non_snake_case)]
pub fn path_trace<S>(sampler: &mut S, mut ray: Ray, scene: &Scene) -> Color
where
    S: Sampler,
{
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

        // TODO: We don't correctly handle inverted normals at the moment. If
        // the normal faces in the same direction as the ray (and we aren't
        // doing transmission), the sampled direction will be in the wrong
        // hemisphere.
        assert_abs_diff_eq!(intersection.normal.magnitude(), 1.0, epsilon = EPSILON);
        assert!(intersection.distance >= 0.0);

        // Both `w_o` and `w_i` should be coming out of the surface
        let w_o = -ray.direction;

        // Normally, contribution from emissive surfaces will be included via
        // `sample_light`, but it will miss two cases:
        // 1. If a camera ray directly hits an emissive surface. The surface
        //    will try to sample itself as a light, but this tends not to work
        //    well.
        // 2. If the material in the previous step returned a delta PDF (i.e. a
        //    specular bounce).
        if is_specular_bounce || bounces == 0 {
            L += beta * intersection.Le(&w_o);
        }

        let path_samples = PathSegmentSamples::from(sampler);

        let surface_sample =
            match intersection
                .material
                .sample(path_samples.material, &w_o, &intersection.normal)
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
        let (light_index, light_pdf) = scene.light_sampler.sample(path_samples.light_index);
        let light = &scene.lights[light_index];
        L +=
            beta * estimate_direct(
                path_samples.light,
                path_samples.brdf,
                &intersection,
                &w_o,
                &light,
                scene,
            ) / light_pdf;

        let cos_theta = surface_sample.w_i.dot(&intersection.normal).abs();
        beta = beta * surface_sample.f * cos_theta;
        if let Pdf::NonDelta(pdf) = surface_sample.pdf {
            beta = beta / pdf;
        }

        // Very naive Russian Roulette
        if bounces > 3 {
            let q: f64 = 0.05_f64.max(1.0 - (beta.r + beta.g + beta.b) * 0.3);
            if path_samples.russian_roulette.take() < q {
                break;
            }
            beta = beta / (1.0 - q);
        }

        ray = Ray::new(intersection.location, surface_sample.w_i);

        bounces += 1;
    }

    assert!(L.r.is_finite());
    assert!(L.g.is_finite());
    assert!(L.b.is_finite());

    L
}
