use std::vec;

use crate::{
    bsdf::BSDF,
    bxdf::{BxDF, Dielectric, Fresnel, SurfaceSample},
    color::Color,
    pdf::Pdf,
    vector::Vector,
};

pub enum Material {
    // TODO: implement lights and remove this
    Emissive { emittance: Color },
    BxDF(BxDF),
    BSDF(BSDF),
}

impl Material {
    pub fn new_emissive(emittance: Color) -> Material {
        Material::Emissive { emittance }
    }
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

    pub fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        match self {
            Material::Emissive { emittance } => Some(SurfaceSample {
                w_i: *w_o,
                f: Color::BLACK,
                pdf: Pdf::Delta,
                Le: *emittance,
            }),
            Material::BxDF(bxdf) => bxdf.sample(w_o, normal),
            Material::BSDF(bsdf) => bsdf.sample(w_o, normal),
        }
    }
}
