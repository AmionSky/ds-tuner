use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct StickOptions {
    /// Deadzone in percentage. (0.0 to 1.0 range)
    pub deadzone: f64,
}
