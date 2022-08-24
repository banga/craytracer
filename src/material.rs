use approx::assert_abs_diff_eq;
use rand::Rng;

use crate::{
    color::Color, constants::EPSILON, intersection::Intersection, ray::Ray,
    sampling::sample_hemisphere, trace, vector::Vector, Scene,
};

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
        normal = normal * -1.0;
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
    fn sample(&self, scene: &Scene, intersection: &Intersection, ray: &Ray, depth: u32) -> Color;
}

// This is a hack to add "lights". TODO: Replace with proper lighting support
pub struct EmissiveMaterial {
    pub emittance: Color,
}

impl Material for EmissiveMaterial {
    fn sample(&self, _scene: &Scene, _intersection: &Intersection, _: &Ray, _depth: u32) -> Color {
        self.emittance
    }
}

pub struct LambertianMaterial {
    pub reflectance: Color,
    pub num_samples: usize,
}

impl Material for LambertianMaterial {
    fn sample(&self, scene: &Scene, intersection: &Intersection, _: &Ray, depth: u32) -> Color {
        let mut irradiance = Color::BLACK;
        for _ in 0..self.num_samples {
            let ray = Ray::new(
                intersection.location,
                sample_hemisphere(&intersection.normal),
            );
            let cos_theta = ray.direction.dot(&intersection.normal);
            irradiance += trace(&ray, scene, depth) * cos_theta;
        }
        irradiance /= self.num_samples as f64;
        irradiance * self.reflectance
    }
}

pub struct Mirror {
    pub reflectance: Color,
}

impl Material for Mirror {
    fn sample(&self, scene: &Scene, intersection: &Intersection, ray: &Ray, depth: u32) -> Color {
        let direction = reflect(&ray.direction, &intersection.normal);
        assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);

        let ray = Ray {
            origin: intersection.location,
            direction,
        };
        trace(&ray, scene, depth) * self.reflectance
    }
}

pub struct Glass {
    pub eta: f64,
    pub transmittance: Color,
}

impl Material for Glass {
    fn sample(&self, scene: &Scene, intersection: &Intersection, ray: &Ray, depth: u32) -> Color {
        let direction = refract(&ray.direction, &intersection.normal, 1.0, self.eta);
        assert_abs_diff_eq!(direction.magnitude(), 1.0, epsilon = EPSILON);

        let ray = Ray {
            origin: intersection.location,
            direction,
        };
        trace(&ray, scene, depth) * self.transmittance
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
