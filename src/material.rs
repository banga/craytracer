use std::vec;

use crate::{
    bsdf::BSDF,
    bxdf::{BxDF, Dielectric, Fresnel, SurfaceSample},
    color::Color,
    geometry::{normal::Normal, vector::Vector},
    pdf::Pdf,
    sampling::samplers::{Sample1d, Sample2d},
    texture::Texture,
};

#[derive(Debug, PartialEq)]
pub enum Material {
    BxDF(BxDF),
    BSDF(BSDF),
}

impl Material {
    pub fn new_matte(reflectance: Texture<Color>, sigma: Texture<f64>) -> Material {
        if sigma.is_zero() {
            Material::BxDF(BxDF::LambertianBRDF { reflectance })
        } else {
            Material::BxDF(BxDF::new_oren_nayyar(reflectance, sigma))
        }
    }
    pub fn new_glass(
        reflectance: Texture<Color>,
        transmittance: Texture<Color>,
        eta: f64,
    ) -> Material {
        Material::BxDF(BxDF::FresnelSpecularBxDF {
            reflectance,
            transmittance,
            eta_i: 1.0,
            eta_t: eta,
        })
    }
    pub fn new_plastic(
        diffuse: Texture<Color>,
        specular: Texture<Color>,
        roughness: Texture<f64>,
    ) -> Material {
        let mut bxdfs: Vec<BxDF> = vec![];
        if !diffuse.is_black() {
            if !roughness.is_zero() {
                bxdfs.push(BxDF::new_oren_nayyar(diffuse, roughness));
            } else {
                bxdfs.push(BxDF::LambertianBRDF {
                    reflectance: diffuse,
                });
            }
        }
        if !specular.is_black() {
            bxdfs.push(BxDF::SpecularBRDF {
                reflectance: specular,
                fresnel: Fresnel::Dielectric(Dielectric {
                    eta_i: 1.0,
                    eta_t: 1.5,
                }),
            });
        }
        Material::BSDF(BSDF { bxdfs })
    }
    pub fn new_metal(eta: Texture<Color>, k: Texture<Color>) -> Material {
        // TODO: Implement microfacet brdf and use here
        Material::BSDF(BSDF {
            bxdfs: vec![BxDF::FresnelConductorBRDF { eta, k }],
        })
    }

    pub fn sample(
        &self,
        (sample_1d, sample_2d): (Sample1d, Sample2d),
        w_o: &Vector,
        normal: &Normal,
        uv: &(f64, f64),
    ) -> Option<SurfaceSample> {
        match self {
            Material::BxDF(bxdf) => bxdf.sample(sample_2d, w_o, normal, uv),
            Material::BSDF(bsdf) => bsdf.sample((sample_1d, sample_2d), w_o, normal, uv),
        }
    }
    pub fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Normal, uv: &(f64, f64)) -> Color {
        match self {
            Material::BxDF(bxdf) => bxdf.f(w_o, w_i, normal, uv),
            Material::BSDF(bsdf) => bsdf.f(w_o, w_i, normal, uv),
        }
    }
    pub fn pdf(&self, w_o: &Vector, w_i: &Vector, normal: &Normal) -> Pdf {
        match self {
            Material::BxDF(bxdf) => bxdf.pdf(w_o, w_i, normal),
            Material::BSDF(bsdf) => bsdf.pdf(w_o, w_i, normal),
        }
    }
}
