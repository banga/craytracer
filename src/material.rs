use crate::{
    color::Color, intersection::Intersection, ray::Ray, sampling::sample_hemisphere, trace, Scene,
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
        let ray = Ray::new(
            intersection.location,
            intersection.normal
                - intersection.normal * (intersection.normal.dot(&ray.direction) * 2.0),
        );
        trace(&ray, scene, depth) * self.reflectance
    }
}
