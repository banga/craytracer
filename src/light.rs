use std::f64::consts::PI;

use approx::assert_abs_diff_eq;

use crate::{color::Color, constants::EPSILON, pdf::Pdf, ray::Ray, vector::Vector};

#[derive(Debug, PartialEq)]
pub enum Light {
    Point {
        origin: Vector,
        intensity: Color, /* Radiant flux per solid angle (W/sr) */
    },
    Distant {
        // Direction the light is arriving from
        direction: Vector,
        intensity: Color, /* Radiant flux per solid angle (W/sr) */
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
    pub fn sample(self: &Self, point: &Vector) -> LightSample {
        match self {
            &Light::Point { origin, intensity } => {
                let op = origin - *point;
                let dist_squared = op.magnitude_squared();
                let dist = dist_squared.sqrt();
                let w_i = op / dist;
                let mut shadow_ray = Ray::new(*point, w_i);
                shadow_ray.update_max_distance(dist);

                LightSample {
                    Li: intensity / dist_squared,
                    w_i,
                    pdf: Pdf::Delta,
                    shadow_ray,
                }
            }
            &Light::Distant {
                direction,
                intensity,
            } => {
                assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);

                // Leave the max distance to infinity, since the light is at qz
                let shadow_ray = Ray::new(*point, direction);

                LightSample {
                    Li: intensity,
                    w_i: direction,
                    pdf: Pdf::Delta,
                    shadow_ray,
                }
            }
        }
    }

    pub fn power(self: &Self) -> Color {
        match self {
            &Light::Point { intensity, .. } => intensity * 4.0 * PI,
            &Light::Distant { intensity, .. } => intensity * PI * WORLD_RADIUS * WORLD_RADIUS,
        }
    }
}
