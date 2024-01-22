use rand::prelude::Distribution;
use rand::Rng;
use rand_distr::{UnitDisc, UnitSphere};

use crate::{
    constants::EPSILON,
    geometry::{normal::Normal, traits::DotProduct, vector::Vector, X, Y, Z},
};

pub fn sample_2d<R>(rng: &mut R) -> (f64, f64)
where
    R: Rng,
{
    (rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
}

pub fn sample_disk<R>(rng: &mut R) -> [f64; 2]
where
    R: Rng,
{
    UnitDisc.sample(rng)
}

pub fn sample_sphere<R>(rng: &mut R) -> Vector
where
    R: Rng,
{
    let [x, y, z] = UnitSphere.sample(rng);
    Vector(x, y, z)
}

pub fn sample_hemisphere<R>(rng: &mut R, normal: &Normal) -> Vector
where
    R: Rng,
{
    let v = sample_sphere(rng);
    if v.dot(normal) > 0.0 {
        v
    } else {
        -v
    }
}

/// Returns the barycentric co-ordinates
pub fn sample_triangle<R>(rng: &mut R) -> [f64; 2]
where
    R: Rng,
{
    let (u, v): (f64, f64) = (rng.gen_range(0.0..=1.0), rng.gen_range(0.0..=1.0));
    let su: f64 = u.sqrt();
    [1.0 - su, v * su]
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

pub fn cosine_sample_hemisphere<R>(rng: &mut R, normal: &Normal) -> Vector
where
    R: Rng,
{
    let normal: Vector = normal.into();

    let (tangent, bitangent) = generate_tangents(&normal);
    let [x, y] = sample_disk(rng);
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
