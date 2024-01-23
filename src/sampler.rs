use crate::geometry::{normal::Normal, traits::DotProduct, vector::Vector};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rand_distr::Uniform;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling#PowerHeuristic
pub fn power_heuristic(n_f: usize, pdf_f: f64, n_g: usize, pdf_g: f64) -> f64 {
    let f = n_f as f64 * pdf_f;
    let g = n_g as f64 * pdf_g;
    (f * f) / (f * f + g * g)
}

pub trait Sampler {
    fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize);
    fn sample_1d(&mut self) -> f64;
    fn sample_2d(&mut self) -> [f64; 2];

    fn sample_disk(&mut self) -> [f64; 2] {
        loop {
            let [u, v] = self.sample_2d();
            // Convert to [-1, 1)^2
            let [u, v] = [2.0 * u - 1.0, 2.0 * v - 1.0];
            if u * u + v * v <= 1.0 {
                return [u, v];
            }
        }
    }

    fn sample_sphere(&mut self) -> Vector {
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

    fn sample_hemisphere(&mut self, normal: &Normal) -> Vector {
        let v = self.sample_sphere();
        if v.dot(normal) > 0.0 {
            v
        } else {
            -v
        }
    }

    /// Returns the barycentric co-ordinates
    fn sample_triangle(&mut self) -> [f64; 2] {
        let [u, v] = self.sample_2d();
        let su: f64 = u.sqrt();
        [1.0 - su, v * su]
    }

    fn cosine_sample_hemisphere(&mut self, normal: &Normal) -> Vector {
        let normal: Vector = normal.into();

        let (tangent, bitangent) = normal.generate_tangents();
        let [x, y] = self.sample_disk();
        let z = (1.0 - x * x - y * y).max(0.0).sqrt();
        let v = tangent * x + bitangent * y + normal * z;
        assert!(v.dot(&normal) >= 0.0);
        v
    }
}

pub struct IndependentSampler {
    seed: u32,
    rng: SmallRng,
    dist: Uniform<f64>,
}

impl IndependentSampler {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rng: SmallRng::from_entropy(),
            dist: Uniform::new(0.0, 1.0),
        }
    }
}

impl Sampler for IndependentSampler {
    fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize) {
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        x.hash(&mut hasher);
        y.hash(&mut hasher);
        sample_index.hash(&mut hasher);
        let hash = hasher.finish();
        self.rng = SmallRng::seed_from_u64(hash);
    }

    /// Returns a value in [0, 1)
    fn sample_1d(&mut self) -> f64 {
        self.rng.sample(self.dist)
    }

    /// Returns a value in [0, 1)^2
    fn sample_2d(&mut self) -> [f64; 2] {
        let mut samples = [0.0; 2];
        self.rng.fill(&mut samples);
        samples
    }
}
