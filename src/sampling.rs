pub mod sampling_fns {
    use super::samplers::Sample2d;
    use crate::geometry::normal::Normal;
    use crate::geometry::traits::DotProduct;
    use crate::geometry::vector::Vector;
    use std::f64::consts::FRAC_PI_2;
    use std::f64::consts::FRAC_PI_4;
    use std::f64::consts::PI;

    /// https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling#PowerHeuristic
    pub fn power_heuristic(n_f: usize, pdf_f: f64, n_g: usize, pdf_g: f64) -> f64 {
        let f = n_f as f64 * pdf_f;
        let g = n_g as f64 * pdf_g;
        (f * f) / (f * f + g * g)
    }

    pub fn sample_disk(sample: Sample2d) -> (f64, f64) {
        let (u, v) = sample.take();
        let (u, v) = (2.0 * u - 1.0, 2.0 * v - 1.0);
        if u == 0.0 || v == 0.0 {
            return (0.0, 0.0);
        }
        let (r, theta) = if u.abs() > v.abs() {
            (u, FRAC_PI_4 * v / u)
        } else {
            (v, FRAC_PI_2 - FRAC_PI_4 * u / v)
        };
        (theta.cos() * r, theta.sin() * r)
    }

    pub fn sample_sphere(sample: Sample2d) -> Vector {
        let (u, v) = sample.take();
        let z = 1.0 - 2.0 * u;
        let r = (1.0 - z.powf(2.0)).max(0.0).sqrt();
        let phi = 2.0 * PI * v;
        let x = r * phi.cos();
        let y = r * phi.sin();
        Vector(x, y, z)
    }

    pub fn sample_hemisphere(s: Sample2d, normal: &Normal) -> Vector {
        let r = sample_sphere(s);
        if r.dot(normal) > 0.0 {
            r
        } else {
            -r
        }
    }

    /// Returns the barycentric co-ordinates
    pub fn sample_triangle(sample: Sample2d) -> (f64, f64) {
        let (u, v) = sample.take();
        let su: f64 = u.sqrt();
        (1.0 - su, v * su)
    }

    pub fn cosine_sample_hemisphere(sample: Sample2d, normal: &Normal) -> Vector {
        let normal: Vector = normal.into();

        let (tangent, bitangent) = normal.generate_tangents();
        let (x, y) = sample_disk(sample);
        let z = (1.0 - x * x - y * y).max(0.0).sqrt();
        let a = tangent * x + bitangent * y + normal * z;
        assert!(a.dot(&normal) >= 0.0);
        a
    }
}

pub mod samplers {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use rand_distr::Uniform;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    // These exist to ensure that avoid accidental copying/reuse of samples.
    // This is done by making sure these structs *do not implement Copy* and
    // can only be used via `take` which consumes `self`.
    pub struct Sample1d(f64);
    impl Sample1d {
        pub fn take(self) -> f64 {
            self.0
        }
    }

    pub struct Sample2d(f64, f64);
    impl Sample2d {
        pub fn take(self) -> (f64, f64) {
            (self.0, self.1)
        }
    }

    pub trait Sampler: Clone {
        fn new(seed: u32) -> Self;
        fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize);
        /// Returns a value in [0, 1)
        fn sample_1d(&mut self) -> Sample1d;
        /// Returns a value in [0, 1)^2
        fn sample_2d(&mut self) -> Sample2d;
    }

    #[derive(Clone)]
    pub struct IndependentSampler {
        seed: u32,
        rng: StdRng,
        dist: Uniform<f64>,
    }

    impl Sampler for IndependentSampler {
        fn new(seed: u32) -> Self {
            Self {
                seed,
                rng: StdRng::seed_from_u64(0),
                dist: Uniform::new(0.0, 1.0),
            }
        }

        fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize) {
            let mut hasher = DefaultHasher::new();
            self.seed.hash(&mut hasher);
            x.hash(&mut hasher);
            y.hash(&mut hasher);
            sample_index.hash(&mut hasher);
            let hash = hasher.finish();
            self.rng = StdRng::seed_from_u64(hash);
        }

        fn sample_1d(&mut self) -> Sample1d {
            Sample1d(self.rng.sample(self.dist))
        }

        fn sample_2d(&mut self) -> Sample2d {
            Sample2d(self.rng.sample(self.dist), self.rng.sample(self.dist))
        }
    }
}
