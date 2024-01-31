use crate::{
    bxdf::{BxDF, SurfaceSample},
    color::Color,
    geometry::{normal::Normal, traits::DotProduct, vector::Vector},
    pdf::Pdf,
    sampling::samplers::{Sample1d, Sample2d},
};

#[derive(Debug)]
pub struct BSDF {
    pub bxdfs: Vec<BxDF>,
}

impl BSDF {
    pub fn sample(
        &self,
        (sample1, sample2): (Sample1d, Sample2d),
        w_o: &Vector,
        normal: &Normal,
        uv: &(f64, f64),
    ) -> Option<SurfaceSample> {
        if self.bxdfs.len() == 0 {
            return None;
        }

        let sample_index = (sample1.take() * self.bxdfs.len() as f64) as usize;
        let bxdf = &self.bxdfs[sample_index];

        let sample = bxdf.sample(sample2, w_o, normal, uv);
        if sample.is_none() {
            return None;
        }

        let sample = sample.unwrap();
        if let Pdf::NonDelta(mut pdf) = sample.pdf {
            let mut f = sample.f;
            self.for_each_relevant_bsdf(w_o, &sample.w_i, normal, |other_idx, other_bxdf| {
                if other_idx != sample_index {
                    f += other_bxdf.f(w_o, &sample.w_i, &normal, uv);
                    if let Pdf::NonDelta(other_pdf) = other_bxdf.pdf(w_o, &sample.w_i, normal) {
                        pdf += other_pdf;
                    }
                }
            });
            Some(SurfaceSample {
                w_i: sample.w_i,
                pdf: Pdf::NonDelta(pdf / self.bxdfs.len() as f64),
                f,
                is_specular: sample.is_specular,
            })
        } else {
            Some(sample)
        }
    }

    fn for_each_relevant_bsdf<F>(&self, w_o: &Vector, w_i: &Vector, normal: &Normal, mut f: F)
    where
        F: FnMut(usize, &BxDF),
    {
        let is_reflecting = w_o.dot(normal) * w_i.dot(normal) > 0.0;
        for (idx, bxdf) in self.bxdfs.iter().enumerate() {
            let is_relevant = if is_reflecting {
                bxdf.has_reflection()
            } else {
                bxdf.has_transmission()
            };
            if is_relevant {
                f(idx, bxdf);
            }
        }
    }

    pub fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Normal, uv: &(f64, f64)) -> Color {
        let mut f = Color::BLACK;
        self.for_each_relevant_bsdf(w_o, w_i, normal, |_, bxdf| {
            f += bxdf.f(w_o, w_i, normal, uv);
        });
        f
    }

    pub fn pdf(&self, w_o: &Vector, w_i: &Vector, normal: &Normal) -> Pdf {
        let mut pdf = 0.0;
        let mut num_matching_bxdfs = 0;
        self.for_each_relevant_bsdf(w_o, w_i, normal, |_, bxdf| {
            match bxdf.pdf(w_o, w_i, normal) {
                Pdf::NonDelta(p) => {
                    pdf += p;
                    num_matching_bxdfs += 1
                }
                Pdf::Delta => {}
            }
        });
        if num_matching_bxdfs > 0 {
            Pdf::NonDelta(pdf / num_matching_bxdfs as f64)
        } else {
            Pdf::Delta
        }
    }
}
