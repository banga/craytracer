use std::vec;

use rand::Rng;

use crate::{
    bsdf::BSDF,
    bxdf::{BxDF, Dielectric, Fresnel, SurfaceSample},
    color::Color,
    geometry::{normal::Normal, vector::Vector},
    pdf::Pdf,
};

#[derive(Debug, PartialEq)]
pub enum Material {
    BxDF(BxDF),
    BSDF(BSDF),
}

impl Material {
    pub fn new_matte(reflectance: Color, sigma: f64) -> Material {
        if sigma == 0.0 {
            Material::BxDF(BxDF::LambertianBRDF { reflectance })
        } else {
            Material::BxDF(BxDF::new_oren_nayyar(reflectance, sigma))
        }
    }
    pub fn new_glass(reflectance: Color, transmittance: Color, eta: f64) -> Material {
        Material::BxDF(BxDF::FresnelSpecularBxDF {
            reflectance,
            transmittance,
            eta_i: 1.0,
            eta_t: eta,
        })
    }
    pub fn new_plastic(diffuse: Color, specular: Color, roughness: f64) -> Material {
        let mut bxdfs: Vec<BxDF> = vec![];
        if !diffuse.is_black() {
            if roughness != 0.0 {
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
    pub fn new_metal(eta: Color, k: Color) -> Material {
        // TODO: Implement microfacet brdf and use here
        Material::BSDF(BSDF {
            bxdfs: vec![BxDF::FresnelConductorBRDF { eta, k }],
        })
    }

    pub fn sample<R>(&self, rng: &mut R, w_o: &Vector, normal: &Normal) -> Option<SurfaceSample>
    where
        R: Rng,
    {
        match self {
            Material::BxDF(bxdf) => bxdf.sample(rng, w_o, normal),
            Material::BSDF(bsdf) => bsdf.sample(rng, w_o, normal),
        }
    }
    pub fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Normal) -> Color {
        match self {
            Material::BxDF(bxdf) => bxdf.f(w_o, w_i, normal),
            Material::BSDF(bsdf) => bsdf.f(w_o, w_i, normal),
        }
    }
    pub fn pdf(&self, w_o: &Vector, w_i: &Vector, normal: &Normal) -> Pdf {
        match self {
            Material::BxDF(bxdf) => bxdf.pdf(w_o, w_i, normal),
            Material::BSDF(bsdf) => bsdf.pdf(w_o, w_i, normal),
        }
    }
}
