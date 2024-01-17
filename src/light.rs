use approx::assert_abs_diff_eq;
use rand::Rng;
use std::{
    f64::consts::{FRAC_1_PI, PI},
    sync::Arc,
};

use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{normal::Normal, point::Point, vector::Vector},
    intersection::PrimitiveIntersection,
    pdf::Pdf,
    ray::Ray,
    sampling::sample_hemisphere,
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

// TODO: This should be computed using the scene's bounds
const WORLD_RADIUS: f64 = 1e6;

impl Light {
    /// Samples the light arriving at a given point from this light source.
    ///
    /// Returns the radiance, the direction from which it is arriving (pointing
    /// to the light source) and the pdf value of sampling that direction.
    pub fn sample<R>(self: &Self, rng: &mut R, point: &Point) -> LightSample
    where
        R: Rng,
    {
        match &self {
            Light::Point { origin, intensity } => {
                let op = *origin - *point;
                let dist_squared = op.magnitude_squared();
                let dist = dist_squared.sqrt();
                let w_i = op / dist;
                let mut shadow_ray = Ray::new(*point, w_i);
                shadow_ray.update_max_distance(dist);

                LightSample {
                    Li: *intensity / dist_squared,
                    w_i,
                    pdf: self.pdf(point, &w_i),
                    shadow_ray,
                }
            }
            Light::Distant {
                direction,
                intensity,
            } => {
                assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);

                // Leave the max distance to infinity, since the light is at qz
                let shadow_ray = Ray::new(*point, *direction);
                let w_i = *direction;

                LightSample {
                    Li: *intensity,
                    w_i,
                    pdf: self.pdf(point, &w_i),
                    shadow_ray,
                }
            }
            Light::Infinite { intensity } => {
                let normal = if rng.gen_bool(0.5) {
                    Normal(1.0, 0.0, 0.0)
                } else {
                    Normal(-1.0, 0.0, 0.0)
                };

                let w_i = sample_hemisphere(rng, &normal);
                let shadow_ray = Ray::new(*point, w_i);

                LightSample {
                    Li: *intensity,
                    w_i,
                    pdf: self.pdf(point, &w_i),
                    shadow_ray,
                }
            }
            Light::Area {
                shape, emittance, ..
            } => {
                // TODO: This sampling approach is not efficient, because we
                // will also sample points on the shape that are not visible
                // from the target point.
                let shape_point = shape.sample(rng);
                let delta = shape_point - *point;
                let distance_squared = delta.magnitude_squared();
                let distance = distance_squared.sqrt();
                let w_i = delta / distance;
                let mut shadow_ray = Ray::new(*point, w_i);
                shadow_ray.update_max_distance(distance - EPSILON);
                LightSample {
                    Li: *emittance,
                    w_i,
                    pdf: self.pdf(point, &w_i),
                    shadow_ray,
                }
            }
        }
    }

    pub fn pdf(self: &Self, point: &Point, w_i: &Vector) -> Pdf {
        match &self {
            Light::Point { .. } => Pdf::Delta,
            Light::Distant { .. } => Pdf::Delta,
            Light::Infinite { .. } => Pdf::NonDelta(FRAC_1_PI / 4.0),
            Light::Area { shape, .. } => shape.pdf(point, w_i),
        }
    }

    /// Light emitted by an area light at the given intersection point in the given direction
    #[allow(non_snake_case)]
    pub fn L(self: &Self, _i: &PrimitiveIntersection, _w_i: &Vector) -> Color {
        match &self {
            Light::Point { .. } => Color::BLACK,
            Light::Distant { .. } => Color::BLACK,
            Light::Infinite { .. } => Color::BLACK,
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

    pub fn power(self: &Self) -> Color {
        match &self {
            Light::Point { intensity, .. } => *intensity * 4.0 * PI,
            Light::Distant { intensity, .. } => *intensity * PI * WORLD_RADIUS * WORLD_RADIUS,
            Light::Infinite { intensity, .. } => *intensity * PI * WORLD_RADIUS * WORLD_RADIUS,
            Light::Area { emittance, shape } => *emittance * PI * shape.area(),
        }
    }
}
