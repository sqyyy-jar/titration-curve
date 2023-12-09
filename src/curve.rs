//! # Notes
//!
//! ```txt
//! pH = -lg(c(H3O+))
//! c(H3O+) = n(H3O+) / Vtotal
//! n(H3O+) =
//! Vtotal = Vm + Vp
//! ```

pub struct Input {
    /// Test solution volume
    pub p_v: f64,
    /// Test solution concentration
    pub p_c: f64,
    /// Measuring solution concentration
    pub m_c: f64,
    /// Measuring solution volumes
    pub m_vs: Vec<f64>,
    // /// pKs (from database, by name)
    // pub pks: f64,
    // /// pKg (from databse, by name)
    // pub pkg: f64,
}

#[derive(PartialEq)]
pub struct Output {
    /// Vtotal
    pub v_total: Vec<f64>,
    /// pH
    pub ph: Vec<f64>,
}

impl Output {
    pub fn max_v(&self) -> f64 {
        self.v_total
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(25.0)
    }
}
