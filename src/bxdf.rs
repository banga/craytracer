use crate::{color::Color, constants::EPSILON, sampling::sample_hemisphere, vector::Vector};
use approx::assert_abs_diff_eq;
use rand::Rng;

#[allow(non_snake_case)]
pub struct SurfaceSample {
    pub w_i: Vector,
    pub f: Color,
    pub Le: Color,
}

pub trait BxDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> SurfaceSample;
}

pub struct LambertianBRDF {
    pub reflectance: Color,
}

impl BxDF for LambertianBRDF {
    fn sample(&self, _w_o: &Vector, normal: &Vector) -> SurfaceSample {
        let w_i = sample_hemisphere(normal);
        SurfaceSample {
            w_i,
            f: self.reflectance,
            Le: Color::BLACK,
        }
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
    // Source: https://pbr-book.org/3ed-2018/Reflection_Models/Specular_Reflection_and_Transmission
    let eta_rel = eta_t / eta_i;
    let eta_rel_2 = eta_rel * eta_rel;
    let k_rel = k / eta_i;
    let k_rel_2 = k_rel * k_rel;

    let cos_theta_2 = cos_theta_i * cos_theta_i;
    let sin_theta_2 = 1.0 - cos_theta_2;
    let t0 = eta_rel_2 - k_rel_2 - Color::WHITE * sin_theta_2;
    let a2_plus_b2 = t0 * t0 + eta_rel_2 * k_rel_2 * 4.0;
    let a = ((a2_plus_b2 + t0) * 0.5).powf(0.5);

    let t1 = a2_plus_b2 + Color::WHITE * cos_theta_2;
    let t2 = a * cos_theta_i * 2.0;
    let r_perpendicular = (t1 - t2) / (t1 + t2);

    let t3 = a2_plus_b2 * cos_theta_2 + Color::WHITE * sin_theta_2 * sin_theta_2;
    let t4 = a * cos_theta_i * sin_theta_2 * 2.0;
    let r_parallel = r_perpendicular * (t3 - t4) / (t3 + t4);

    (r_parallel * r_parallel + r_perpendicular * r_perpendicular) * 0.5
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
    fn sample(&self, w_o: &Vector, normal: &Vector) -> SurfaceSample {
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
        let f = self.reflectance * fresnel / cos_theta_i.abs();

        SurfaceSample {
            w_i,
            f,
            Le: Color::BLACK,
        }
    }
}

pub struct SpecularBTDF {
    pub transmittance: Color,
    pub eta_i: f64,
    pub eta_t: f64,
}

impl BxDF for SpecularBTDF {
    fn sample(&self, w_o: &Vector, normal: &Vector) -> SurfaceSample {
        let cos_theta_i = -w_o.dot(normal);

        if let Some(w_i) = refract(&w_o, &normal, cos_theta_i, self.eta_i, self.eta_t) {
            assert_abs_diff_eq!(w_i.magnitude(), 1.0, epsilon = EPSILON);

            let fresnel = fresnel_dielectric(self.eta_i, self.eta_t, cos_theta_i);
            let f = self.transmittance * (1.0 - fresnel) / cos_theta_i.abs();

            SurfaceSample {
                w_i,
                f,
                Le: Color::BLACK,
            }
        } else {
            SurfaceSample {
                w_i: *w_o,
                f: Color::BLACK,
                Le: Color::BLACK,
            }
        }
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
    fn sample(&self, w_o: &Vector, normal: &Vector) -> SurfaceSample {
        let cos_theta_i = -w_o.dot(&normal);
        let fresnel_reflectance = fresnel_dielectric(self.eta_i, self.eta_t, cos_theta_i);

        // Here, `f` doesn't need to be multiplied by the fresnel reflectance
        // value because that is the `pdf` of the function and will be in the
        // denominator of the monte-carlo estimator and cancel out
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < fresnel_reflectance {
            SurfaceSample {
                w_i: reflect(w_o, normal),
                f: self.reflectance / cos_theta_i.abs(),
                Le: Color::BLACK,
            }
        } else {
            if let Some(w_i) = refract(w_o, normal, cos_theta_i, self.eta_i, self.eta_t) {
                SurfaceSample {
                    w_i,
                    f: self.transmittance / cos_theta_i.abs(),
                    Le: Color::BLACK,
                }
            } else {
                SurfaceSample {
                    w_i: *w_o,
                    f: Color::BLACK,
                    Le: Color::BLACK,
                }
            }
        }
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
