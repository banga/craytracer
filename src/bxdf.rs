use crate::{
    color::Color,
    constants::EPSILON,
    geometry::{normal::Normal, traits::DotProduct, vector::Vector},
    pdf::Pdf,
    sampling::{samplers::Sample2d, sampling_fns::cosine_sample_hemisphere},
};
use approx::assert_abs_diff_eq;
use std::f64::consts::FRAC_1_PI;

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SurfaceSample {
    pub w_i: Vector,
    pub f: Color,
    pub pdf: Pdf,
    pub is_specular: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq)]
pub enum BxDF {
    LambertianBRDF {
        reflectance: Color,
    },
    OrenNayyarBRDF {
        reflectance: Color,
        A: f64,
        B: f64,
    },
    FresnelConductorBRDF {
        eta: Color,
        k: Color,
    },
    SpecularBRDF {
        reflectance: Color,
        fresnel: Fresnel,
    },
    SpecularBTDF {
        transmittance: Color,
        eta_i: f64,
        eta_t: f64,
    },
    FresnelSpecularBxDF {
        reflectance: Color,
        transmittance: Color,
        eta_i: f64,
        eta_t: f64,
    },
}

impl BxDF {
    pub fn new_oren_nayyar(reflectance: Color, sigma: f64) -> BxDF {
        let sigma = sigma.to_radians();
        let sigma_2 = sigma * sigma;
        BxDF::OrenNayyarBRDF {
            reflectance,
            A: 1.0 - sigma_2 / (2.0 * (sigma_2 + 0.33)),
            B: 0.45 * sigma_2 / (sigma_2 + 0.09),
        }
    }

    pub fn has_reflection(&self) -> bool {
        match self {
            BxDF::LambertianBRDF { .. } => true,
            BxDF::OrenNayyarBRDF { .. } => true,
            BxDF::FresnelConductorBRDF { .. } => true,
            BxDF::SpecularBRDF { .. } => true,
            BxDF::SpecularBTDF { .. } => false,
            BxDF::FresnelSpecularBxDF { .. } => true,
        }
    }

    pub fn has_transmission(&self) -> bool {
        match self {
            BxDF::LambertianBRDF { .. } => false,
            BxDF::OrenNayyarBRDF { .. } => false,
            BxDF::FresnelConductorBRDF { .. } => false,
            BxDF::SpecularBRDF { .. } => false,
            BxDF::SpecularBTDF { .. } => true,
            BxDF::FresnelSpecularBxDF { .. } => true,
        }
    }

    /// Samples the BRDF given a surface `normal` and an outgoing direction for
    /// light `w_o`. The sample includes `w_i` the sampled incoming direction of
    /// light, `f` the value of the BRDF at this sample and `pdf` the value of
    /// the probability density function at this sample.
    pub fn sample(&self, sample: Sample2d, w_o: &Vector, normal: &Normal) -> Option<SurfaceSample> {
        match self {
            BxDF::LambertianBRDF { .. } => {
                let mut w_i = cosine_sample_hemisphere(sample, normal);
                // Make sure w_i is in the same hemisphere as w_o
                if normal.dot(w_o) < 0.0 {
                    w_i = -w_i;
                }
                Some(SurfaceSample {
                    w_i,
                    f: self.f(w_o, &w_i, normal),
                    pdf: self.pdf(w_o, &w_i, normal),
                    is_specular: false,
                })
            }
            BxDF::OrenNayyarBRDF { .. } => {
                let mut w_i = cosine_sample_hemisphere(sample, normal);
                // Make sure w_i is in the same hemisphere as w_o
                if normal.dot(w_o) < 0.0 {
                    w_i = -w_i;
                }
                Some(SurfaceSample {
                    w_i,
                    f: self.f(w_o, &w_i, normal),
                    pdf: self.pdf(w_o, &w_i, normal),
                    is_specular: false,
                })
            }
            BxDF::FresnelConductorBRDF { eta, k } => {
                let w_i = reflect(&w_o, &normal);
                assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);

                let cos_theta_i = w_o.dot(normal).abs();
                let fresnel = fresnel_conductor(&Color::WHITE, eta, k, cos_theta_i);
                Some(SurfaceSample {
                    w_i,
                    f: fresnel / cos_theta_i,
                    pdf: self.pdf(w_o, &w_i, normal),
                    is_specular: true,
                })
            }
            BxDF::SpecularBRDF {
                reflectance,
                fresnel,
            } => {
                let w_i = reflect(&w_o, &normal);
                assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);

                let cos_theta_i = w_o.dot(normal).abs();
                let fresnel = match fresnel {
                    Fresnel::Dielectric(dielectric) => {
                        Color::WHITE
                            * fresnel_dielectric(dielectric.eta_i, dielectric.eta_t, cos_theta_i)
                    }
                    Fresnel::Conductor(conductor) => fresnel_conductor(
                        &conductor.eta_i,
                        &conductor.eta_t,
                        &conductor.k,
                        cos_theta_i,
                    ),
                };
                Some(SurfaceSample {
                    w_i,
                    f: *reflectance * fresnel / cos_theta_i.abs(),
                    pdf: self.pdf(w_o, &w_i, normal),
                    is_specular: true,
                })
            }
            BxDF::SpecularBTDF {
                transmittance,
                eta_i,
                eta_t,
            } => {
                let cos_theta_i = w_o.dot(normal).abs();

                if let Some(w_i) = refract(&w_o, &normal, cos_theta_i, *eta_i, *eta_t) {
                    assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);
                    let fresnel = fresnel_dielectric(*eta_i, *eta_t, cos_theta_i);
                    Some(SurfaceSample {
                        w_i,
                        f: *transmittance * (1.0 - fresnel) / cos_theta_i,
                        pdf: self.pdf(w_o, &w_i, normal),
                        is_specular: true,
                    })
                } else {
                    None
                }
            }
            BxDF::FresnelSpecularBxDF {
                reflectance,
                transmittance,
                eta_i,
                eta_t,
            } => {
                let cos_theta_i = w_o.dot(normal);
                let fresnel_reflectance = fresnel_dielectric(*eta_i, *eta_t, cos_theta_i);

                if sample.take().0 < fresnel_reflectance {
                    Some(SurfaceSample {
                        w_i: reflect(w_o, normal),
                        f: *reflectance * fresnel_reflectance / cos_theta_i.abs(),
                        pdf: Pdf::NonDelta(fresnel_reflectance),
                        is_specular: true,
                    })
                } else {
                    if let Some(w_i) = refract(w_o, normal, cos_theta_i, *eta_i, *eta_t) {
                        Some(SurfaceSample {
                            w_i,
                            f: *transmittance * (1.0 - fresnel_reflectance) / cos_theta_i.abs(),
                            pdf: Pdf::NonDelta(1.0 - fresnel_reflectance),
                            is_specular: true,
                        })
                    } else {
                        None
                    }
                }
            }
        }
    }

    /// Returns the value of the BRDF given outgoing and incoming directions for
    /// light `w_o` and `w_i`
    pub fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Normal) -> Color {
        match self {
            BxDF::LambertianBRDF { reflectance } => {
                if normal.same_hemisphere(w_o, w_i) {
                    *reflectance * FRAC_1_PI
                } else {
                    Color::BLACK
                }
            }
            BxDF::OrenNayyarBRDF { reflectance, A, B } => {
                if normal.same_hemisphere(w_o, w_i) {
                    let cos_theta_i = w_i.dot(normal).abs();
                    let cos_theta_o = w_o.dot(normal).abs();

                    let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
                    let sin_theta_o = (1.0 - cos_theta_o * cos_theta_o).max(0.0).sqrt();
                    let max_cos = if sin_theta_i > 1e-4 && sin_theta_o > 1e-4 {
                        // TODO: Calculating the tangent can be avoided by
                        // transforming to shading space first
                        let tangent = normal.generate_tangents().0;
                        let cos_phi_i = w_i.dot(&tangent).abs();
                        let cos_phi_o = w_o.dot(&tangent).abs();
                        let sin_phi_i = (1.0 - cos_phi_i * cos_phi_i).sqrt();
                        let sin_phi_o = (1.0 - cos_phi_o * cos_phi_o).sqrt();
                        (cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o).max(0.0)
                    } else {
                        0.0
                    };

                    let (sin_alpha, tan_beta) = if cos_theta_i > cos_theta_o {
                        // theta_i <= theta_o
                        (sin_theta_o, sin_theta_i / cos_theta_i)
                    } else {
                        (sin_theta_i, sin_theta_o / cos_theta_o)
                    };

                    *reflectance * (A + B * max_cos * sin_alpha * tan_beta) * FRAC_1_PI
                } else {
                    Color::BLACK
                }
            }
            BxDF::FresnelConductorBRDF { .. } => Color::BLACK,
            BxDF::SpecularBRDF { .. } => Color::BLACK,
            BxDF::SpecularBTDF { .. } => Color::BLACK,
            BxDF::FresnelSpecularBxDF { .. } => Color::BLACK,
        }
    }

    /// Return the value of the probability density function of sampling this
    /// BRDF in the incoming direction `w_i`
    pub fn pdf(&self, _w_o: &Vector, w_i: &Vector, normal: &Normal) -> Pdf {
        match self {
            BxDF::LambertianBRDF { .. } => {
                let cos_theta = w_i.dot(normal).abs();
                Pdf::NonDelta(FRAC_1_PI * cos_theta)
            }
            BxDF::OrenNayyarBRDF { .. } => {
                let cos_theta = w_i.dot(normal).abs();
                Pdf::NonDelta(FRAC_1_PI * cos_theta)
            }
            BxDF::FresnelConductorBRDF { .. } => Pdf::Delta,
            BxDF::SpecularBRDF { .. } => Pdf::Delta,
            BxDF::SpecularBTDF { .. } => Pdf::Delta,
            BxDF::FresnelSpecularBxDF { .. } => Pdf::Delta,
        }
    }
}

pub fn reflect(direction: &Vector, normal: &Normal) -> Vector {
    let normal: Vector = normal.into();
    normal * (normal.dot(direction) * 2.0) - *direction
}

pub fn refract(
    direction: &Vector,
    normal: &Normal,
    cos_theta_i: f64,
    eta_i: f64,
    eta_t: f64,
) -> Option<Vector> {
    let normal: Vector = normal.into();
    let (normal, eta_relative, cos_theta) = if cos_theta_i.is_sign_negative() {
        (-normal, eta_i / eta_t, -cos_theta_i)
    } else {
        (normal, eta_t / eta_i, cos_theta_i)
    };

    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    if sin_theta > eta_relative {
        return None;
    }

    let r_perpendicular = (normal * cos_theta - *direction) / eta_relative;
    let r_parallel = normal * -(1.0 - r_perpendicular.dot(&r_perpendicular)).sqrt();
    Some(r_perpendicular + r_parallel)
}

#[derive(Debug, PartialEq)]

pub struct Dielectric {
    pub eta_i: f64,
    pub eta_t: f64,
}

#[derive(Debug, PartialEq)]

pub struct Conductor {
    pub eta_i: Color,
    pub eta_t: Color,
    pub k: Color,
}

#[derive(Debug, PartialEq)]

pub enum Fresnel {
    Dielectric(Dielectric),
    Conductor(Conductor),
}

fn fresnel_dielectric(eta_i: f64, eta_t: f64, cos_theta_i: f64) -> f64 {
    let (cos_theta_i, eta_i, eta_t) = if cos_theta_i.is_sign_negative() {
        (-cos_theta_i, eta_t, eta_i)
    } else {
        (cos_theta_i, eta_i, eta_t)
    };

    let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).sqrt();
    let sin_theta_t = eta_i / eta_t * sin_theta_i;
    if sin_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).sqrt();
    let r_parallel =
        (eta_t * cos_theta_i - eta_i * cos_theta_t) / (eta_t * cos_theta_i + eta_i * cos_theta_t);
    let r_perpendicular =
        (eta_i * cos_theta_i - eta_t * cos_theta_t) / (eta_i * cos_theta_i + eta_t * cos_theta_t);
    (r_parallel * r_parallel + r_perpendicular * r_perpendicular) * 0.5
}

fn fresnel_conductor(eta_i: &Color, eta_t: &Color, k: &Color, cos_theta_i: f64) -> Color {
    assert!(cos_theta_i >= 0.0);
    // Source: https://pbr-book.org/3ed-2018/Reflection_Models/Specular_Reflection_and_Transmission
    let eta_rel = *eta_t / *eta_i;
    let eta_rel_2 = eta_rel * eta_rel;
    let k_rel = *k / *eta_i;
    let k_rel_2 = k_rel * k_rel;

    let cos_theta_2 = cos_theta_i * cos_theta_i;
    let sin_theta_2 = 1.0 - cos_theta_2;
    let t0 = eta_rel_2 - k_rel_2 - Color::WHITE * sin_theta_2;
    let a2_plus_b2 = (t0 * t0 + eta_rel_2 * k_rel_2 * 4.0).powf(0.5);
    let a = ((a2_plus_b2 + t0) * 0.5).powf(0.5);

    let t1 = a2_plus_b2 + Color::WHITE * cos_theta_2;
    let t2 = a * cos_theta_i * 2.0;
    let r_perpendicular = (t1 - t2) / (t1 + t2);

    let t3 = a2_plus_b2 * cos_theta_2 + Color::WHITE * sin_theta_2 * sin_theta_2;
    let t4 = a * cos_theta_i * sin_theta_2 * 2.0;
    let r_parallel = r_perpendicular * (t3 - t4) / (t3 + t4);

    (r_parallel * r_parallel + r_perpendicular * r_perpendicular) * 0.5
}
