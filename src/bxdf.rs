use std::f64::consts::FRAC_1_PI;

use crate::{
    color::Color, constants::EPSILON, pdf::Pdf, sampling::sample_hemisphere, vector::Vector,
};
use approx::assert_abs_diff_eq;
use rand::Rng;

#[allow(non_snake_case)]
pub struct SurfaceSample {
    pub w_i: Vector,
    pub f: Color,
    pub pdf: Pdf,
    // TODO: Handle lighting separately
    pub Le: Color,
}

pub trait BxDF: Sync + Send {
    fn has_reflection(&self) -> bool;
    fn has_transmission(&self) -> bool;
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample>;
    fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Vector) -> Color;
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> Pdf;
}

pub struct LambertianBRDF {
    reflectance: Color,
}

impl LambertianBRDF {
    pub fn new(reflectance: Color) -> LambertianBRDF {
        LambertianBRDF { reflectance }
    }
}

impl BxDF for LambertianBRDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let w_i = sample_hemisphere(normal);
        Some(SurfaceSample {
            w_i,
            f: self.f(w_o, &w_i, normal),
            pdf: self.pdf(w_o, &w_i),
            Le: Color::BLACK,
        })
    }
    fn f(&self, _w_o: &Vector, _w_i: &Vector, _normal: &Vector) -> Color {
        self.reflectance * FRAC_1_PI
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::NonDelta(FRAC_1_PI)
    }
    fn has_reflection(&self) -> bool {
        true
    }
    fn has_transmission(&self) -> bool {
        false
    }
}

#[allow(non_snake_case)]
pub struct OrenNayyarBRDF {
    reflectance: Color,
    A: f64,
    B: f64,
}

#[allow(non_snake_case)]
impl OrenNayyarBRDF {
    pub fn new(reflectance: Color, sigma: f64) -> OrenNayyarBRDF {
        let sigma = sigma.to_radians();
        let sigma_2 = sigma * sigma;
        let A = 1.0 - sigma_2 / (2.0 * (sigma_2 + 0.33));
        let B = 0.45 * sigma_2 / (sigma_2 + 0.09);
        OrenNayyarBRDF { reflectance, A, B }
    }
}

impl BxDF for OrenNayyarBRDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let w_i = sample_hemisphere(normal);

        Some(SurfaceSample {
            w_i,
            f: self.f(w_o, &w_i, normal),
            pdf: self.pdf(w_o, &w_i),
            Le: Color::BLACK,
        })
    }
    fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Vector) -> Color {
        let cos_theta_i = w_i.dot(normal);
        assert!(cos_theta_i >= 0.0);
        let cos_theta_o = w_o.dot(normal).abs();

        let sin_theta_i = (1.0 - cos_theta_i).sqrt();
        let sin_theta_o = (1.0 - cos_theta_o).sqrt();
        let max_cos = (cos_theta_i * cos_theta_o + sin_theta_i * sin_theta_o).max(0.0);

        let (sin_alpha, tan_beta) = if cos_theta_i > cos_theta_o {
            // theta_i <= theta_o
            (sin_theta_o, sin_theta_i / cos_theta_i)
        } else {
            (sin_theta_i, sin_theta_o / cos_theta_o)
        };

        self.reflectance * (self.A + self.B * max_cos * sin_alpha * tan_beta) * FRAC_1_PI
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::NonDelta(FRAC_1_PI)
    }
    fn has_reflection(&self) -> bool {
        true
    }
    fn has_transmission(&self) -> bool {
        false
    }
}

fn reflect(direction: &Vector, normal: &Vector) -> Vector {
    *direction - *normal * (normal.dot(direction) * 2.0)
}

fn refract(
    direction: &Vector,
    normal: &Vector,
    cos_theta_i: f64,
    eta_i: f64,
    eta_t: f64,
) -> Option<Vector> {
    let (normal, eta_relative, cos_theta) = if cos_theta_i.is_sign_negative() {
        (-*normal, eta_i / eta_t, -cos_theta_i)
    } else {
        (*normal, eta_t / eta_i, cos_theta_i)
    };

    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    if sin_theta > eta_relative {
        return None;
    }

    let r_perpendicular = (*direction + normal * cos_theta) / eta_relative;
    let r_parallel = normal * -(1.0 - r_perpendicular.dot(&r_perpendicular)).sqrt();
    Some(r_perpendicular + r_parallel)
}

pub struct Dielectric {
    pub eta_i: f64,
    pub eta_t: f64,
}

pub struct Conductor {
    pub eta_i: Color,
    pub eta_t: Color,
    pub k: Color,
}

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

fn fresnel_conductor(eta_i: Color, eta_t: Color, k: Color, cos_theta_i: f64) -> Color {
    assert!(cos_theta_i >= 0.0);
    // Source: https://pbr-book.org/3ed-2018/Reflection_Models/Specular_Reflection_and_Transmission
    let eta_rel = eta_t / eta_i;
    let eta_rel_2 = eta_rel * eta_rel;
    let k_rel = k / eta_i;
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

pub struct FresnelConductorBRDF {
    eta: Color,
    k: Color,
}

impl FresnelConductorBRDF {
    pub fn new(eta: Color, k: Color) -> FresnelConductorBRDF {
        FresnelConductorBRDF { eta, k }
    }
}

impl BxDF for FresnelConductorBRDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let w_i = reflect(&w_o, &normal);
        assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);

        let cos_theta_i = -w_o.dot(normal);
        let fresnel = fresnel_conductor(Color::WHITE, self.eta, self.k, cos_theta_i);
        Some(SurfaceSample {
            w_i,
            f: fresnel / cos_theta_i.abs(),
            pdf: self.pdf(w_o, &w_i),
            Le: Color::BLACK,
        })
    }
    fn f(&self, _w_o: &Vector, _w_i: &Vector, _normal: &Vector) -> Color {
        Color::BLACK
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::Delta
    }
    fn has_reflection(&self) -> bool {
        true
    }
    fn has_transmission(&self) -> bool {
        false
    }
}

pub struct SpecularBRDF {
    reflectance: Color,
    fresnel: Fresnel,
}

impl SpecularBRDF {
    pub fn new(reflectance: Color, fresnel: Fresnel) -> SpecularBRDF {
        SpecularBRDF {
            reflectance,
            fresnel,
        }
    }
}

impl BxDF for SpecularBRDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let w_i = reflect(&w_o, &normal);
        assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);

        let cos_theta_i = -w_o.dot(normal);
        let fresnel = match &self.fresnel {
            Fresnel::Dielectric(dielectric) => {
                Color::WHITE * fresnel_dielectric(dielectric.eta_i, dielectric.eta_t, cos_theta_i)
            }
            Fresnel::Conductor(conductor) => {
                // TODO: Test this, probably needs to take cos_theta_i.abs()
                fresnel_conductor(conductor.eta_i, conductor.eta_t, conductor.k, cos_theta_i)
            }
        };
        Some(SurfaceSample {
            w_i,
            f: self.reflectance * fresnel / cos_theta_i.abs(),
            pdf: self.pdf(w_o, &w_i),
            Le: Color::BLACK,
        })
    }
    fn f(&self, _w_o: &Vector, _w_i: &Vector, _normal: &Vector) -> Color {
        Color::BLACK
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::Delta
    }
    fn has_reflection(&self) -> bool {
        true
    }
    fn has_transmission(&self) -> bool {
        false
    }
}

pub struct SpecularBTDF {
    pub transmittance: Color,
    pub eta_i: f64,
    pub eta_t: f64,
}

impl BxDF for SpecularBTDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let cos_theta_i = -w_o.dot(normal);

        if let Some(w_i) = refract(&w_o, &normal, cos_theta_i, self.eta_i, self.eta_t) {
            assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);
            let fresnel = fresnel_dielectric(self.eta_i, self.eta_t, cos_theta_i);
            Some(SurfaceSample {
                w_i,
                f: self.transmittance * (1.0 - fresnel) / cos_theta_i.abs(),
                pdf: self.pdf(w_o, &w_i),
                Le: Color::BLACK,
            })
        } else {
            None
        }
    }
    fn f(&self, _w_o: &Vector, _w_i: &Vector, _normal: &Vector) -> Color {
        Color::BLACK
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::Delta
    }
    fn has_reflection(&self) -> bool {
        false
    }
    fn has_transmission(&self) -> bool {
        true
    }
}

pub struct FresnelSpecularBxDF {
    reflectance: Color,
    transmittance: Color,
    eta_i: f64,
    eta_t: f64,
}

impl FresnelSpecularBxDF {
    pub fn new(
        reflectance: Color,
        transmittance: Color,
        eta_i: f64,
        eta_t: f64,
    ) -> FresnelSpecularBxDF {
        FresnelSpecularBxDF {
            reflectance,
            transmittance,
            eta_i,
            eta_t,
        }
    }
}

impl BxDF for FresnelSpecularBxDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let cos_theta_i = -w_o.dot(&normal);
        let fresnel_reflectance = fresnel_dielectric(self.eta_i, self.eta_t, cos_theta_i);

        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < fresnel_reflectance {
            Some(SurfaceSample {
                w_i: reflect(w_o, normal),
                f: self.reflectance * fresnel_reflectance / cos_theta_i.abs(),
                pdf: Pdf::NonDelta(fresnel_reflectance),
                Le: Color::BLACK,
            })
        } else {
            if let Some(w_i) = refract(w_o, normal, cos_theta_i, self.eta_i, self.eta_t) {
                Some(SurfaceSample {
                    w_i,
                    f: self.transmittance * (1.0 - fresnel_reflectance) / cos_theta_i.abs(),
                    pdf: Pdf::NonDelta(1.0 - fresnel_reflectance),
                    Le: Color::BLACK,
                })
            } else {
                None
            }
        }
    }
    fn f(&self, _w_o: &Vector, _w_i: &Vector, _normal: &Vector) -> Color {
        Color::BLACK
    }
    fn pdf(&self, _w_o: &Vector, _w_i: &Vector) -> Pdf {
        Pdf::Delta
    }
    fn has_reflection(&self) -> bool {
        true
    }
    fn has_transmission(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

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
        let direction = &Vector::new(1, -1, 0).normalized();
        let normal = Vector::Y.normalized();
        assert_abs_diff_eq!(
            refract(&direction, &normal, -direction.dot(&normal), 1.0, 1.0).unwrap(),
            Vector::new(1, -1, 0).normalized()
        );
    }
}
