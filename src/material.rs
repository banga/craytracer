use crate::{
    intersection::Intersection,
    ray::Ray,
    sampling::sample_hemisphere,
    trace,
    vector::{Color, Vector},
    Scene,
};

pub trait Material: Sync + Send {
    fn sample(&self, scene: &Scene, intersection: &Intersection, ray: &Ray, depth: u32) -> Color;
}

pub struct LambertianMaterial {
    pub reflectance: Color,
    pub num_samples: usize,
}

impl Material for LambertianMaterial {
    fn sample(&self, scene: &Scene, intersection: &Intersection, _: &Ray, depth: u32) -> Color {
        let mut irradiance = Color::NULL;
        for _ in 0..self.num_samples {
            let ray = Ray::new(
                intersection.location,
                sample_hemisphere(&intersection.normal),
            );
            let cos_theta = ray.direction.dot(&intersection.normal);
            irradiance += trace(&ray, scene, depth) * cos_theta;
        }
        irradiance /= self.num_samples as f64;
        Vector(
            irradiance.x() * self.reflectance.x(),
            irradiance.y() * self.reflectance.y(),
            irradiance.z() * self.reflectance.z(),
        )
    }
}

pub struct Mirror {}

impl Material for Mirror {
    fn sample(&self, scene: &Scene, intersection: &Intersection, ray: &Ray, depth: u32) -> Color {
        let ray = Ray::new(
            intersection.location,
            intersection.normal
                - intersection.normal * (intersection.normal.dot(&ray.direction) * 2.0),
        );
        trace(&ray, scene, depth)
    }
}
