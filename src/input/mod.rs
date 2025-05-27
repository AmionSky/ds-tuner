mod options;
mod util;

pub use options::*;

use self::util::*;
use glam::DVec2;

pub fn gen_stick_lut(options: &StickOptions) -> Vec<u16> {
    let mut values = vec![0; 256 * 256];

    for y in 0..=255 {
        for x in 0..=255 {
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
    } else if options.deadzone_rescale {
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
