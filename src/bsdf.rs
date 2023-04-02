use std::sync::Arc;

use rand::Rng;

use crate::{
    bxdf::{BxDF, SurfaceSample},
    pdf::Pdf,
    vector::Vector,
};

pub struct BSDF {
    pub bxdfs: Vec<Arc<dyn BxDF>>,
}

impl BSDF {
    pub fn sample(&self, w_o: &Vector, normal: &Vector) -> Option<SurfaceSample> {
        let is_reflecting = w_o.dot(&normal) < 0.0;
        let relevant_bxdfs: Vec<&Arc<dyn BxDF>> = self
            .bxdfs
            .iter()
            .filter(|b| {
                if is_reflecting {
                    b.has_reflection()
                } else {
                    b.has_transmission()
                }
            })
            .collect();
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
}
