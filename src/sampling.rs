use rand::Rng;
use rand_distr::Uniform;

use crate::{
    constants::EPSILON,
    geometry::{normal::Normal, traits::DotProduct, vector::Vector, X, Y, Z},
};

#[derive(Debug, PartialEq)]
pub struct Sampler<R> {
    rng: R,
    dist: Uniform<f64>,
}

impl<R> Sampler<R>
where
    R: Rng,
{
    pub fn new(rng: R) -> Self {
        Sampler {
            rng,
            dist: Uniform::new(0.0, 1.0),
        }
    }

    /// Returns a value in [0, 1)
    pub fn sample_1d(&mut self) -> f64 {
        self.rng.sample(self.dist)
    }

    /// Returns a value in [0, 1)^2
    pub fn sample_2d(&mut self) -> [f64; 2] {
        let mut samples = [0.0; 2];
        self.rng.fill(&mut samples);
        samples
    }

    pub fn sample_disk(&mut self) -> [f64; 2] {
        loop {
            let [u, v] = self.sample_2d();
            // Convert to [-1, 1)^2
            let [u, v] = [2.0 * u - 1.0, 2.0 * v - 1.0];
            if u * u + v * v <= 1.0 {
                return [u, v];
            }
        }
    }

    pub fn sample_sphere(&mut self) -> Vector {
        loop {
            let [u, v] = self.sample_2d();
            // Convert to [-1, 1)
            let [u, v] = [2.0 * u - 1.0, 2.0 * v - 1.0];
            let sum = u * u + v * v;
            if sum < 1.0 {
                let factor = 2.0 * (1.0 - sum).sqrt();
                return Vector(u * factor, v * factor, 1.0 - 2.0 * sum);
            }
        }
    }

    pub fn sample_hemisphere(&mut self, normal: &Normal) -> Vector {
        let v = self.sample_sphere();
        if v.dot(normal) > 0.0 {
            v
        } else {
            -v
        }
    }

    /// Returns the barycentric co-ordinates
    pub fn sample_triangle(&mut self) -> [f64; 2] {
        let [u, v] = self.sample_2d();
        let su: f64 = u.sqrt();
        [1.0 - su, v * su]
    }

    pub fn cosine_sample_hemisphere(&mut self, normal: &Normal) -> Vector {
        let normal: Vector = normal.into();

        let (tangent, bitangent) = generate_tangents(&normal);
        let [x, y] = self.sample_disk();
        let z = (1.0 - x * x - y * y).max(0.0).sqrt();
        let v = tangent * x + bitangent * y + normal * z;
        assert!(v.dot(&normal) >= 0.0);
        v
    }
}

fn generate_tangents(vector: &Vector) -> (Vector, Vector) {
    let other = if vector.x().abs() < EPSILON {
        X
    } else if vector.y().abs() < EPSILON {
        Y
    } else {
        Z
    };
    let tangent = vector.cross(&other).normalized();
    let bitangent = vector.cross(&tangent).normalized();
    (tangent, bitangent)
}

/// https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling#PowerHeuristic
pub fn power_heuristic(n_f: usize, pdf_f: f64, n_g: usize, pdf_g: f64) -> f64 {
    let f = n_f as f64 * pdf_f;
    let g = n_g as f64 * pdf_g;
    (f * f) / (f * f + g * g)
}
