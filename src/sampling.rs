use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    constants::EPSILON,
    geometry::{normal::Normal, traits::DotProduct, vector::Vector},
};

pub fn sample_2d() -> (f64, f64) {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    (uniform.sample(&mut rng), uniform.sample(&mut rng))
}

pub fn sample_disk() -> (f64, f64) {
    // Concentric sampling: https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations#ConcentricSampleDisk
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    let x: f64 = uniform.sample(&mut rng);
    let y: f64 = uniform.sample(&mut rng);
    if x == 0.0 && y == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if x.abs() > y.abs() {
        (x, FRAC_PI_4 * (y / x))
    } else {
        (y, FRAC_PI_2 - FRAC_PI_4 * (x / y))
    };

    (r * theta.cos(), r * theta.sin())
}

pub fn sample_hemisphere(normal: &Normal) -> Vector {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    loop {
        let v = Vector(
            uniform.sample(&mut rng),
            uniform.sample(&mut rng),
            uniform.sample(&mut rng),
        );
        let magnitude_squared = v.magnitude_squared();
        if magnitude_squared <= 1.0 {
            if v.dot(normal) > 0.0 {
                return v / magnitude_squared.sqrt();
            } else {
                return -v / magnitude_squared.sqrt();
            }
        }
    }
}

fn generate_tangents(vector: &Vector) -> (Vector, Vector) {
    let other = if vector.x().abs() < EPSILON {
        Vector::X
    } else if vector.y().abs() < EPSILON {
        Vector::Y
    } else {
        Vector::Z
    };
    let tangent = vector.cross(&other).normalized();
    let bitangent = vector.cross(&tangent).normalized();
    (tangent, bitangent)
}

pub fn cosine_sample_hemisphere(normal: &Normal) -> Vector {
    let normal: Vector = normal.into();

    let (tangent, bitangent) = generate_tangents(&normal);
    let (x, y) = sample_disk();
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    let v = tangent * x + bitangent * y + normal * z;
    assert!(v.dot(&normal) >= 0.0);
    v
}

/// https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling#PowerHeuristic
pub fn power_heuristic(n_f: usize, pdf_f: f64, n_g: usize, pdf_g: f64) -> f64 {
    let f = n_f as f64 * pdf_f;
    let g = n_g as f64 * pdf_g;
    (f * f) / (f * f + g * g)
}
