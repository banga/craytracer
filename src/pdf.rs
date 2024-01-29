#[derive(Debug)]
pub enum Pdf {
    NonDelta(f64),
    // Dirac-delta distribution, which usually requires special handling
    Delta,
}
