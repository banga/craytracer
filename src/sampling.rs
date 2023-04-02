use rand::{distributions::Uniform, prelude::Distribution};
use rand_distr::UnitSphere;

use crate::vector::Vector;

pub fn sample_2d() -> (f64, f64) {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    (uniform.sample(&mut rng), uniform.sample(&mut rng))
}

pub fn sample_hemisphere(normal: &Vector) -> Vector {
    let mut rng = rand::thread_rng();
    let [x, y, z] = UnitSphere.sample(&mut rng);
    let v = Vector(x, y, z);
    if v.dot(normal) > 0.0 {
        return v;
    } else {
        return -v;
    }
}
