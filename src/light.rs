use std::f64::consts::{FRAC_1_PI, PI};

use approx::assert_abs_diff_eq;
use rand_distr::{Distribution, Uniform};

use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{normal::Normal, point::Point, vector::Vector},
    pdf::Pdf,
    ray::Ray,
    sampling::sample_hemisphere,
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
    pub fn sample(self: &Self, point: &Point) -> LightSample {
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
                    pdf: self.pdf(&w_i),
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
                let w_i = direction;

                LightSample {
                    Li: intensity,
                    w_i,
                    pdf: self.pdf(&w_i),
                    shadow_ray,
                }
            }
            &Light::Infinite { intensity } => {
                let mut rng = rand::thread_rng();
                let uniform = Uniform::new_inclusive(-1.0, 1.0);
                let x = uniform.sample(&mut rng);
                let normal = if x > 0.0 {
                    Normal(1.0, 0.0, 0.0)
                } else {
                    Normal(-1.0, 0.0, 0.0)
                };

                let w_i = sample_hemisphere(&normal);
                let shadow_ray = Ray::new(*point, w_i);

                LightSample {
                    Li: intensity,
                    w_i,
                    pdf: self.pdf(&w_i),
                    shadow_ray,
                }
            }
        }
    }

    pub fn pdf(self: &Self, _w_i: &Vector) -> Pdf {
        match self {
            &Light::Point { .. } => Pdf::Delta,
            &Light::Distant { .. } => Pdf::Delta,
            &Light::Infinite { .. } => Pdf::NonDelta(FRAC_1_PI / 4.0),
        }
    }

    #[allow(non_snake_case)]
    pub fn Le(self: &Self, _w_i: &Vector) -> Color {
        match self {
            &Light::Point { .. } => Color::BLACK,
            &Light::Distant { .. } => Color::BLACK,
            &Light::Infinite { intensity } => intensity,
        }
    }

    pub fn power(self: &Self) -> Color {
        match self {
            &Light::Point { intensity, .. } => intensity * 4.0 * PI,
            &Light::Distant { intensity, .. } => intensity * PI * WORLD_RADIUS * WORLD_RADIUS,
            &Light::Infinite { intensity, .. } => intensity * PI * WORLD_RADIUS * WORLD_RADIUS,
        }
    }
}
