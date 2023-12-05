//! # Notes
//!
//! pH = 1/2 * (pKs - lg(0.1))

pub struct Input {
    /// Measured solution volume
    pub m_v: f64,
    /// Measured solution concentration
    pub m_c: f64,
    /// Test solution volume
    pub p_v: f64,
    /// Test solution concentration
    pub p_c: f64,
    /// pKs (from database, by name)
    pub pks: f64,
    /// pKg (from databse, by name)
    pub pkg: f64,
}

pub struct Output {
    /// (V, V_total)
    pub v_total: Vec<(f64, f64)>,
    pub ph: Vec<f64>,
}
