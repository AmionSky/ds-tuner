use super::util::*;
use glam::DVec2;
use serde::Deserialize;

//
// Options
//

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct StickOptions {
    /// Deadzone radius.
    pub deadzone: f64,
    /// Rescale the input to start from center after deadzone.
    pub rescale: bool,
    /// Limits the max radius.
    pub limit: Option<f64>,
}

impl Default for StickOptions {
    fn default() -> Self {
        Self {
            deadzone: 0.0,
            rescale: true,
            limit: None,
        }
    }
}

impl StickOptions {
    pub fn gen_lut(&self) -> Vec<u16> {
        gen_lut(self)
    }
}

//
// LUT Generator
//

fn gen_lut(options: &StickOptions) -> Vec<u16> {
    let mut values = vec![0; 256 * 256];

    for y in 0..=u8::MAX {
        for x in 0..=u8::MAX {
            let mut input = DVec2::new(to_scaled(x), to_scaled(y));

            apply_deadzone(&mut input, options);
            apply_limit(&mut input, options);

            let index = x as usize + y as usize * 256;
            values[index] = to_merged(input.x, input.y);
        }
    }

    values
}

fn apply_deadzone(input: &mut DVec2, options: &StickOptions) {
    let len_squared = input.length_squared();
    // Check if input length is less than the deadzone amount
    if len_squared < sq(options.deadzone) {
        *input = DVec2::ZERO;
    } else if options.rescale {
        if let Some(dir) = input.try_normalize() {
            let mut len = len_squared.sqrt();
            // Scale the length to take into account the deadzone.
            // Use unscaled lenght if it's longer than 1.0
            len = deadzone_scale(len, options.deadzone).min(len);
            *input = dir * len;
        }
    }
}

fn apply_limit(input: &mut DVec2, options: &StickOptions) {
    if let Some(limit) = options.limit {
        if let Some(dir) = input.try_normalize() {
            let len = input.length();
            *input = dir * len.min(limit);
        }
    }
}

//
// Utility
//

const RAW_CENTER: f64 = u8::MAX as f64 / 2.0;

/// Converts raw value into -1.0 to 1.0 range.
fn to_scaled(raw: u8) -> f64 {
    (RAW_CENTER - raw as f64) * (1.0 / RAW_CENTER)
}

/// Converts scaled value into raw value.
fn to_raw(scaled: f64) -> u8 {
    u8::MAX - (RAW_CENTER + (scaled * RAW_CENTER)).round() as u8
}

/// Creates a single u16 containing 2 raw values from scaled values.
fn to_merged(x: f64, y: f64) -> u16 {
    let x = to_raw(x) as u16;
    let y = to_raw(y) as u16;
    (x & 0x00FF) + ((y << 8) & 0xFF00)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_conversion() {
        for x in 0..=u8::MAX {
            assert_eq!(x, to_raw(to_scaled(x)));
        }
    }

    #[test]
    fn check_center() {
        assert_eq!(127, to_raw(0.0));
    }

    #[test]
    fn check_scaled_center() {
        fn check(value: u8) {
            let value = to_scaled(value);
            assert!(value.abs() < 0.01, "value = {value}");
        }

        check(127);
        check(128);
    }

    #[test]
    fn check_merged() {
        let x = 37;
        let y = 185;
        let merged = to_merged(to_scaled(x), to_scaled(y));

        assert_eq!(x, (merged & 0xFF) as u8);
        assert_eq!(y, ((merged >> 8) & 0xFF) as u8);
    }
}
