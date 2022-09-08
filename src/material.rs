use std::{sync::Arc, vec};

use crate::{
    bsdf::BSDF,
    bxdf::{
        BxDF, Dielectric, Fresnel, FresnelSpecularBxDF, LambertianBRDF, OrenNayyarBRDF,
        SpecularBRDF, SurfaceSample,
    },
    color::Color,
    pdf::Pdf,
    vector::Vector,
};

pub trait Material: Sync + Send {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample>;
}

pub struct EmissiveMaterial {
    pub emittance: Color,
}

impl Material for EmissiveMaterial {
    fn sample(&self, w_o: &Vector, _normal: &Vector) -> Option<SurfaceSample> {
        Some(SurfaceSample {
            w_i: *w_o,
            f: Color::BLACK,
            pdf: Pdf::Delta,
            Le: self.emittance,
        })
    }
}

pub enum MatteMaterial {
    Lambertian(LambertianBRDF),
    OrenNayyar(OrenNayyarBRDF),
}

impl MatteMaterial {
    pub fn new(reflectance: Color, sigma: f64) -> MatteMaterial {
        if sigma == 0.0 {
            MatteMaterial::Lambertian(LambertianBRDF::new(reflectance))
        } else {
            MatteMaterial::OrenNayyar(OrenNayyarBRDF::new(reflectance, sigma))
        }
    }
}

impl Material for MatteMaterial {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        match self {
            MatteMaterial::Lambertian(brdf) => brdf.sample(w_o, normal),
            MatteMaterial::OrenNayyar(brdf) => brdf.sample(w_o, normal),
        }
    }
}

pub struct MirorMaterial {
    brdf: SpecularBRDF,
}

// TODO: Not really a mirror like material, just exposing it for testing
impl MirorMaterial {
    pub fn new(reflectance: Color, eta: f64) -> MirorMaterial {
        MirorMaterial {
            brdf: SpecularBRDF::new(
                reflectance,
                Fresnel::Dielectric(Dielectric {
                    eta_i: 1.0,
                    eta_t: eta,
                }),
            ),
        }
    }
}

impl Material for MirorMaterial {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        self.brdf.sample(w_o, normal)
    }
}

pub struct GlassMaterial {
    bxdf: FresnelSpecularBxDF,
}

impl GlassMaterial {
    pub fn new(reflectance: Color, transmittance: Color, eta: f64) -> GlassMaterial {
        GlassMaterial {
            bxdf: FresnelSpecularBxDF::new(reflectance, transmittance, 1.0, eta),
        }
    }
}

impl Material for GlassMaterial {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        self.bxdf.sample(w_o, normal)
    }
}

pub struct PlasticMaterial {
    bsdf: BSDF,
}

impl PlasticMaterial {
    // TODO: Implement microfacet brdf and use here
    pub fn new(diffuse: Color, specular: Color, roughness: f64) -> PlasticMaterial {
        let mut bxdfs: Vec<Arc<dyn BxDF>> = vec![];
        if !diffuse.is_black() {
            if roughness != 0.0 {
                bxdfs.push(Arc::new(OrenNayyarBRDF::new(diffuse, roughness)));
            } else {
                bxdfs.push(Arc::new(LambertianBRDF::new(diffuse)));
            }
        }
        if !specular.is_black() {
            bxdfs.push(Arc::new(SpecularBRDF::new(
                specular,
                Fresnel::Dielectric(Dielectric {
                    eta_i: 1.0,
                    eta_t: 1.5,
                }),
            )));
        }
        PlasticMaterial {
            bsdf: BSDF { bxdfs },
        }
    }
}

impl Material for PlasticMaterial {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        self.bsdf.sample(w_o, normal)
    }
}
