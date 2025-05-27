use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct StickOptions {
    /// Deadzone radius.
    pub deadzone: f64,
    /// Rescale the output to start from center after deadzone.
    pub deadzone_rescale: bool,
    /// Limits the max radius.
    pub limit: Option<f64>,
}

impl Default for StickOptions {
    fn default() -> Self {
        Self {
            deadzone: 0.0,
            deadzone_rescale: true,
            limit: None,
        }
    }
}
