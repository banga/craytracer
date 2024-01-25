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
        // TODO: This fails when using SobolSampler
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
        fn num_samples(&self) -> usize;
        fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize);
        /// Returns a value in [0, 1)
        fn sample_1d(&mut self) -> Sample1d;
        /// Returns a value in [0, 1)^2
        fn sample_2d(&mut self) -> Sample2d;
    }

    #[derive(Clone)]
    pub struct IndependentSampler {
        seed: usize,
        rng: StdRng,
        dist: Uniform<f64>,
        num_samples: usize,
    }

    impl IndependentSampler {
        pub fn new(seed: usize, num_samples: usize) -> Self {
            Self {
                seed,
                rng: StdRng::seed_from_u64(0),
                dist: Uniform::new(0.0, 1.0),
                num_samples,
            }
        }
    }

    impl Sampler for IndependentSampler {
        fn num_samples(&self) -> usize {
            self.num_samples
        }
        fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize) {
            let mut hasher = DefaultHasher::new();
            self.seed.hash(&mut hasher);
            x.hash(&mut hasher);
            y.hash(&mut hasher);
            // Ideally, we would keep all samples in the same sequence by only
            // hashing (x, y) and then advancing the sequence by a large
            // multiple of sample_index. But Rng doesn't support that and we are
            // generating completely independent samples anyway.
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

    // Divides each dimension into equal slots and samples the center of each
    // slot. This is not a good sampler, because samples from each dimension are
    // correlated. A stratified sampler could be built from this if we could
    // remove this correlation by randomly permuting the order in which these
    // samples are drawn per dimension. It's here mainly to help visualize other
    // sampling functions that use a sampler.
    #[derive(Clone)]
    pub struct UniformSampler {
        num_x_samples: usize,
        num_y_samples: usize,
        sample_index: usize,
    }

    impl UniformSampler {
        pub fn new(num_x_samples: usize, num_y_samples: usize) -> Self {
            Self {
                num_x_samples,
                num_y_samples,
                sample_index: 0,
            }
        }
    }

    impl Sampler for UniformSampler {
        fn num_samples(&self) -> usize {
            self.num_x_samples * self.num_y_samples
        }

        fn start_pixel(&mut self, _x: usize, _y: usize, sample_index: usize) {
            self.sample_index = sample_index;
        }

        fn sample_1d(&mut self) -> Sample1d {
            let sample =
                (self.sample_index as f64 + 0.5) / (self.num_x_samples * self.num_y_samples) as f64;
            Sample1d(sample)
        }

        fn sample_2d(&mut self) -> Sample2d {
            let x = self.sample_index % self.num_x_samples;
            let y = self.sample_index / self.num_x_samples;

            let sample_x = (x as f64 + 0.5) / self.num_x_samples as f64;
            let sample_y = (y as f64 + 0.5) / self.num_y_samples as f64;
            Sample2d(sample_x, sample_y)
        }
    }

    #[derive(Clone)]
    pub struct SobolSampler {
        seed: usize,
        num_samples: usize,

        hash: u32,
        sample_index: u32,
        dimension: u32,
    }

    impl SobolSampler {
        pub fn new(seed: usize, num_samples: usize) -> Self {
            SobolSampler {
                seed,
                num_samples,
                hash: 0,
                sample_index: 0,
                dimension: 0,
            }
        }
    }

    impl Sampler for SobolSampler {
        fn num_samples(&self) -> usize {
            self.num_samples
        }

        fn start_pixel(&mut self, x: usize, y: usize, sample_index: usize) {
            let mut hasher = DefaultHasher::new();
            self.seed.hash(&mut hasher);
            x.hash(&mut hasher);
            y.hash(&mut hasher);
            self.hash = hasher.finish() as u32;

            self.sample_index = sample_index as u32;
            self.dimension = 0;
        }

        fn sample_1d(&mut self) -> Sample1d {
            let sample = sobol_burley::sample(self.sample_index, self.dimension, self.hash);
            self.dimension += 1;
            Sample1d(sample as f64)
        }

        fn sample_2d(&mut self) -> Sample2d {
            let sample_x = sobol_burley::sample(self.sample_index, self.dimension, self.hash);
            self.dimension += 1;
            let sample_y = sobol_burley::sample(self.sample_index, self.dimension, self.hash);
            self.dimension += 1;
            Sample2d(sample_x as f64, sample_y as f64)
        }
    }
}
