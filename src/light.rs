use approx::assert_abs_diff_eq;
use std::{
    f64::consts::{FRAC_1_PI, PI},
    sync::Arc,
};

use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{point::Point, vector::Vector},
    intersection::PrimitiveIntersection,
    n,
    pdf::Pdf,
    ray::Ray,
    sampling::{
        samplers::{Sample1d, Sample2d},
        sampling_fns::sample_hemisphere,
    },
    shape::Shape,
};

#[derive(Debug, PartialEq)]
pub enum Light {
    Point {
        origin: Point,
        intensity: Color, /* Radiant flux per solid angle (W/sr) */
    },
    Distant {
        // Direction the light is arriving from
        direction: Vector,
        intensity: Color, /* Radiant flux per solid angle (W/sr) */
    },
    Infinite {
        // TODO: add texture for the intensity
        intensity: Color, /* Radiant flux per solid angle (W/sr) */
    },
    Area {
        shape: Arc<Shape>,
        emittance: Color,
    },
}

#[allow(non_snake_case)]
pub struct LightSample {
    pub Li: Color,
    pub w_i: Vector,
    pub pdf: Pdf,
    pub shadow_ray: Ray,
}

impl Light {
    /// Samples the light arriving at a given point from this light source.
    ///
    /// Returns the radiance, the direction from which it is arriving (pointing
    /// to the light source) and the pdf value of sampling that direction.
    #[allow(non_snake_case)]
    pub fn sample_Li(
        self: &Self,
        (sample_1d, sample_2d): (Sample1d, Sample2d),
        intersection: &PrimitiveIntersection,
    ) -> LightSample {
        match &self {
            Light::Point { origin, intensity } => {
                let op = *origin - intersection.location;
                let dist_squared = op.magnitude_squared();
                let dist = dist_squared.sqrt();
                let w_i = op / dist;
                let mut shadow_ray = Ray::new(intersection.location, w_i);
                shadow_ray.update_max_distance(dist);

                LightSample {
                    Li: *intensity / dist_squared,
                    w_i,
                    pdf: self.pdf_Li(intersection, &w_i),
                    shadow_ray,
                }
            }
            Light::Distant {
                direction,
                intensity,
            } => {
                assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);

                // Leave the max distance to infinity, since the light is at qz
                let shadow_ray = Ray::new(intersection.location, *direction);
                let w_i = *direction;

                LightSample {
                    Li: *intensity,
                    w_i,
                    pdf: self.pdf_Li(intersection, &w_i),
                    shadow_ray,
                }
            }
            Light::Infinite { intensity } => {
                let normal = if sample_1d.take() < 0.5 {
                    n!(1, 0, 0)
                } else {
                    n!(-1, 0, 0)
                };

                let w_i = sample_hemisphere(sample_2d, &normal);
                let shadow_ray = Ray::new(intersection.location, w_i);

                LightSample {
                    Li: *intensity,
                    w_i,
                    pdf: self.pdf_Li(intersection, &w_i),
                    shadow_ray,
                }
            }
            Light::Area {
                shape, emittance, ..
            } => {
                // TODO: The way sample_from is implemented, it can sample a
                // point that is not actually visible from the intersection. It
                // returns a pdf of 0.0 in such cases, which must be handled
                // where it is used.
                let (shape_point, w_i, pdf) = shape.sample_from(sample_2d, intersection);
                let distance = (shape_point - intersection.location).magnitude();
                let mut shadow_ray = Ray::new(intersection.location, w_i);
                shadow_ray.update_max_distance(distance - EPSILON);
                return LightSample {
                    Li: *emittance,
                    w_i,
                    pdf,
                    shadow_ray,
                };
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn pdf_Li(self: &Self, intersection: &PrimitiveIntersection, w_i: &Vector) -> Pdf {
        match &self {
            Light::Point { .. } => Pdf::Delta,
            Light::Distant { .. } => Pdf::Delta,
            Light::Infinite { .. } => Pdf::NonDelta(FRAC_1_PI / 4.0),
            Light::Area { shape, .. } => shape.pdf_from(intersection, w_i),
        }
    }

    /// Light emitted by an area light at the given intersection point in the given direction
    #[allow(non_snake_case)]
    pub fn L(self: &Self, _i: &PrimitiveIntersection, _w_i: &Vector) -> Color {
        match &self {
            Light::Point { .. } => unreachable!(),
            Light::Distant { .. } => unreachable!(),
            Light::Infinite { .. } => unreachable!(),
            // Area lights are currently assumed to be two-sided. If sidedness
            // needs to be added, we can check which side the normal at the
            // intersection lies w.r.t. w_i
            Light::Area { emittance, .. } => *emittance,
        }
    }

    /// Light emitted along a direction that did not hit the scene
    #[allow(non_snake_case)]
    pub fn Le(self: &Self, _w_i: &Vector) -> Color {
        match &self {
            Light::Point { .. } => Color::BLACK,
            Light::Distant { .. } => Color::BLACK,
            Light::Infinite { intensity } => *intensity,
            Light::Area { .. } => Color::BLACK,
        }
    }

    pub fn power(self: &Self, world_radius: f64) -> Color {
        match &self {
            Light::Point { intensity, .. } => *intensity * 4.0 * PI,
            Light::Distant { intensity, .. } => *intensity * PI * world_radius * world_radius,
            Light::Infinite { intensity, .. } => *intensity * PI * world_radius * world_radius,
            Light::Area { emittance, shape } => *emittance * PI * shape.area(),
        }
    }
}

/// Samples lights in proportion to their power
#[derive(Debug, PartialEq)]
pub struct LightSampler {
    cdfs: Vec<f64>,
}

impl LightSampler {
    pub fn new(lights: &Vec<Arc<Light>>, world_radius: f64) -> Self {
        let mut total_power = 0.0;
        let cdfs: Vec<f64> = lights
            .iter()
            .map(|light| {
                let power = light.power(world_radius);
                let power_avg = (power.r + power.g + power.b) / 3.0;
                total_power += power_avg;
                total_power
            })
            .collect();
        let cdfs = cdfs.iter().map(|power| power / total_power).collect();
        Self { cdfs }
    }

    // TODO: Try the "AliasTable" method to do this in constant time
    pub fn sample(&self, sample: Sample1d) -> (usize, f64) {
        let u = sample.take();
        let idx = self
            .cdfs
            .binary_search_by(|probe| probe.total_cmp(&u))
            .unwrap_or_else(|idx| idx);
        let pdf = self.pdf(idx);
        (idx, pdf)
    }

    pub fn pdf(&self, light_idx: usize) -> f64 {
        if light_idx > 0 {
            self.cdfs[light_idx] - self.cdfs[light_idx - 1]
        } else {
            self.cdfs[light_idx]
        }
    }
}
