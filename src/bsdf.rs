use rand::Rng;

use crate::{
    bxdf::{BxDF, SurfaceSample},
    color::Color,
    pdf::Pdf,
    vector::Vector,
};

#[derive(Debug, PartialEq)]
pub struct BSDF {
    pub bxdfs: Vec<BxDF>,
}

impl BSDF {
    // TODO: This isn't quite right. The caller should be able to specify which
    // bxdf types to use.
    fn get_relevant_bxdfs(&self, w_o: &Vector, normal: &Vector) -> Vec<&BxDF> {
        let is_reflecting = w_o.dot(&normal) < 0.0;
        self.bxdfs
            .iter()
            .filter(|b| {
                if is_reflecting {
                    b.has_reflection()
                } else {
                    b.has_transmission()
                }
            })
            .collect()
    }

    pub fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let relevant_bxdfs = self.get_relevant_bxdfs(w_o, normal);
        if relevant_bxdfs.len() == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let sample_index = rng.gen_range(0..relevant_bxdfs.len());
        let bxdf = &relevant_bxdfs[sample_index];

        let sample = bxdf.sample(w_o, normal);
        if sample.is_none() {
            return None;
        }

        let sample = sample.unwrap();
        if let Pdf::NonDelta(mut pdf) = sample.pdf {
            let mut f = sample.f;
            for (index, other_bxdf) in relevant_bxdfs.iter().enumerate() {
                if index != sample_index {
                    f += other_bxdf.f(w_o, &sample.w_i, &normal);
                    if let Pdf::NonDelta(other_pdf) = other_bxdf.pdf(w_o, &sample.w_i, normal) {
                        pdf += other_pdf;
                    }
                }
            }
            Some(SurfaceSample {
                w_i: sample.w_i,
                pdf: Pdf::NonDelta(pdf / relevant_bxdfs.len() as f64),
                f,
                Le: sample.Le,
            })
        } else {
            Some(sample)
        }
    }

    pub fn f(&self, w_o: &Vector, w_i: &Vector, normal: &Vector) -> Color {
        let mut f = Color::BLACK;
        for bxdf in self.get_relevant_bxdfs(w_o, normal) {
            f += bxdf.f(w_o, w_i, normal);
        }
        f
    }

    pub fn pdf(&self, w_o: &Vector, w_i: &Vector, normal: &Vector) -> Pdf {
        let mut pdf = 0.0;
        let mut num_matching_bxdfs = 0;
        for bxdf in self.get_relevant_bxdfs(w_o, normal) {
            match bxdf.pdf(w_o, w_i, normal) {
                Pdf::NonDelta(p) => {
                    pdf += p;
                    num_matching_bxdfs += 1
                }
                Pdf::Delta => {}
            }
        }
        if num_matching_bxdfs > 0 {
            Pdf::NonDelta(pdf / num_matching_bxdfs as f64)
        } else {
            Pdf::Delta
        }
    }
}
