use rand::{distributions::Uniform, prelude::Distribution};

use crate::vector::Vector;

pub fn sample_2d() -> (f64, f64) {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    (uniform.sample(&mut rng), uniform.sample(&mut rng))
}

pub fn sample_hemisphere(normal: &Vector) -> Vector {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new_inclusive(-1.0, 1.0);
    loop {
        let v = Vector(
            uniform.sample(&mut rng),
            uniform.sample(&mut rng),
            uniform.sample(&mut rng),
        );
        let magnitude_squared = v.dot(&v);
        if magnitude_squared <= 1.0 {
            if v.dot(normal) > 0.0 {
                return v / magnitude_squared.sqrt();
            } else {
                return -v / magnitude_squared.sqrt();
            }
        }
    }
}
