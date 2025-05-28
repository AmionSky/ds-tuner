use super::util::*;
use serde::Deserialize;

//
// Options
//

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct TriggerOptions {
    /// Deadzone percentage.
    pub deadzone: f64,
    /// Rescale the value to start from 0.0 after deadzone.
    pub rescale: bool,
}

impl Default for TriggerOptions {
    fn default() -> Self {
        Self {
            deadzone: 0.0,
            rescale: true,
        }
    }
}

impl TriggerOptions {
    pub fn gen_lut(&self) -> Vec<u8> {
        gen_lut(self)
    }
}

//
// LUT Generator
//

fn gen_lut(options: &TriggerOptions) -> Vec<u8> {
    let mut values = vec![0; 256];

    for index in 0..=u8::MAX {
        let mut value = to_scaled(index);

        apply_deadzone(&mut value, options);

        values[index as usize] = to_raw(value);
    }

    values
}

fn apply_deadzone(input: &mut f64, options: &TriggerOptions) {
    if *input < options.deadzone {
        *input = 0.0;
    } else if options.rescale {
        *input = deadzone_scale(*input, options.deadzone);
    }
}

//
// Utility
//

/// Converts raw value into 0.0 to 1.0 range.
fn to_scaled(raw: u8) -> f64 {
    raw as f64 / u8::MAX as f64
}

/// Converts scaled value into raw value.
fn to_raw(scaled: f64) -> u8 {
    (scaled * u8::MAX as f64).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_conversion() {
        for v in 0..=u8::MAX {
            assert_eq!(v, to_raw(to_scaled(v)));
        }
    }
}
