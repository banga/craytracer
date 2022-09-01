use approx::assert_abs_diff_eq;
use rand::Rng;

use crate::{color::Color, constants::EPSILON, sampling::sample_hemisphere, vector::Vector};

fn reflect(direction: &Vector, normal: &Vector) -> Vector {
    *direction - *normal * (normal.dot(direction) * 2.0)
}

fn refract(direction: &Vector, normal: &Vector, eta_i: f64, eta_t: f64) -> Vector {
    let mut normal = *normal;
    let mut cos_theta = -direction.dot(&normal);
    let mut eta_relative = eta_t / eta_i;
    if cos_theta < 0.0 {
        cos_theta = -cos_theta;
        eta_relative = eta_i / eta_t;
        normal = -normal;
    }

    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    if sin_theta > eta_relative {
        return reflect(direction, &normal);
    }

    // Schlick approximation of Fresnel reflectance
    let mut rng = rand::thread_rng();
    let r0 = ((1.0 - eta_relative) / (1.0 + eta_relative)).powf(2.0);
    let reflectance = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);
    if rng.gen::<f64>() < reflectance {
        return reflect(direction, &normal);
    }

    let r_perpendicular = (*direction + normal * cos_theta) / eta_relative;
    let r_parallel = normal * -(1.0 - r_perpendicular.dot(&r_perpendicular)).sqrt();
    r_perpendicular + r_parallel
}

pub trait Material: Sync + Send {
    fn sample(
        &self,
        wo: &Vector,
        normal: &Vector,
    ) -> (
        /* wi */ Vector,
        /* f */ Color,
        /* Le */ Color,
    );
}

pub struct EmissiveMaterial {
    pub emittance: Color,
}

impl Material for EmissiveMaterial {
    fn sample(&self, wo: &Vector, _normal: &Vector) -> (Vector, Color, Color) {
        (*wo, Color::BLACK, self.emittance)
    }
}

pub struct LambertianMaterial {
    pub reflectance: Color,
}

impl Material for LambertianMaterial {
    fn sample(&self, _wo: &Vector, normal: &Vector) -> (Vector, Color, Color) {
        let wi = sample_hemisphere(normal);
        let cos_theta = wi.dot(normal);
        (wi, self.reflectance * cos_theta, Color::BLACK)
    }
}

pub struct Mirror {
    pub reflectance: Color,
}

impl Material for Mirror {
    fn sample(&self, wo: &Vector, normal: &Vector) -> (Vector, Color, Color) {
        let wi = reflect(&wo, &normal);
        assert_abs_diff_eq!(wi.magnitude(), 1.0, epsilon = EPSILON);

        (wi, self.reflectance, Color::BLACK)
    }
}

pub struct Glass {
    pub eta: f64,
    pub transmittance: Color,
}

impl Material for Glass {
    fn sample(&self, wo: &Vector, normal: &Vector) -> (Vector, Color, Color) {
        let wi = refract(&wo, &normal, 1.0, self.eta);
        assert_abs_diff_eq!(wi.magnitude(), 1.0, epsilon = EPSILON);

        (wi, self.transmittance, Color::BLACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::*;

    #[test]
    fn reflect_test() {
        assert_abs_diff_eq!(
            reflect(&Vector(1.0, -1.0, 0.0).normalized(), &Vector(0.0, 1.0, 0.0)),
            Vector(1.0, 1.0, 0.0).normalized()
        );
    }

    #[test]
    fn refract_test() {
        assert_abs_diff_eq!(
            refract(
                &Vector(1.0, -1.0, 0.0).normalized(),
                &Vector(0.0, 1.0, 0.0),
                1.0,
                1.0
            ),
            Vector(1.0, -1.0, 0.0).normalized()
        );
    }
}
