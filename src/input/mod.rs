mod util;

use glam::DVec2;
use util::*;

pub fn gen_stick_lut(deadzone: f64) -> Vec<u16> {
    let mut values = vec![0; 256 * 256];

    for y in 0..=255 {
        for x in 0..=255 {
            let mut input = DVec2::new(to_scaled(x), to_scaled(y));

            apply_deadzone(&mut input, deadzone);

            let index = x as usize + y as usize * 256;
            values[index] = to_merged(input.x, input.y);
        }
    }

    values
}

fn apply_deadzone(input: &mut DVec2, amount: f64) {
    let len_squared = input.length_squared();
    // Check if input length is less than the deadzone amount
    if len_squared < sq(amount) {
        *input = DVec2::ZERO;
    } else if let Some(dir) = input.try_normalize() {
        let mut len = len_squared.sqrt();
        // Scale the length to take into account the deadzone.
        // Use unscaled lenght if it's longer than 1.0
        len = deadzone_scale(len, amount).min(len);
        *input = dir * len;
    }
}

/// Scale length outside of deadzone to full 0.0 to 1.0+ range
const fn deadzone_scale(len: f64, deadzone: f64) -> f64 {
    (len - deadzone).max(0.0) * (1.0 / (1.0 - deadzone))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_deadzone_scale() {
        const DEADZONE: f64 = 0.2;
        assert_eq!(0.000, deadzone_scale(0.0, DEADZONE));
        assert_eq!(0.000, deadzone_scale(0.2, DEADZONE));
        assert_eq!(1.000, deadzone_scale(1.0, DEADZONE));
        assert_eq!(0.375, deadzone_scale(0.5, DEADZONE));
    }
}
