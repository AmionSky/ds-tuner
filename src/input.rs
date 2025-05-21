use glam::DVec2;

const CENTER: f64 = 127.5;

const fn vector_from_raw(x: u8, y: u8) -> DVec2 {
    DVec2::new(to_scaled(x), to_scaled(y))
}

pub fn gen_input() -> [u16; 256 * 256] {
    let mut values = [0u16; 256 * 256];

    for iy in 0..=255 {
        for ix in 0..=255 {
            let index = ix + iy * 256;

            let input = vector_from_raw(ix as u8, iy as u8);
            let output = modify(input);

            values[index] = to_merged(output.x, output.y);
        }
    }

    /*
    for y in 0..256 {
        for x in 0..256 {
            let index = x + y * 256;
            let value = values[index];

            print!(
                "X{} Y{} = X{} Y{} |",
                index & 0xFF,
                (index >> 8) & 0xFF,
                value & 0xFF,
                (value >> 8) & 0xFF,
            );
        }
        println!()
    }
    */

    values
}

fn modify(input: DVec2) -> DVec2 {
    let mut output = DVec2::ZERO;

    let deadzone = 0.2;

    // deadzone
    if input.length_squared() < sq(deadzone) {
        return output;
    } else if let Some(dir) = input.try_normalize() {
        let len = deadzone_scale(input.length(), deadzone);
        output = dir * len;
    }

    output
}

/// Square (n * n)
const fn sq(n: f64) -> f64 {
    n * n
}

/// Converts raw value into -1.0 to 1.0 range
const fn to_scaled(raw: u8) -> f64 {
    (CENTER - raw as f64) * (1.0 / CENTER)
}

/// Converts scaled value into raw value
fn to_raw(scaled: f64) -> u8 {
    255 - (CENTER + (scaled * CENTER)).round() as u8
}

fn to_merged(x: f64, y: f64) -> u16 {
    let x = to_raw(x) as u16;
    let y = to_raw(y) as u16;
    (x & 0x00FF) + ((y << 8) & 0xFF00)
}

/// Scale length outside of deadzone to full 0.0 to 1.0 range
const fn deadzone_scale(len: f64, deadzone: f64) -> f64 {
    (len - deadzone).max(0.0) * (1.0 / (1.0 - deadzone))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_scaled_centered() {
        let val = to_scaled(127);
        assert!(val.abs() < 0.01, "value = {val}");
    }

    #[test]
    fn check_conversion() {
        for x in 0..=255 {
            let c = to_raw(to_scaled(x));
            assert_eq!(x, c);
        }
    }

    #[test]
    fn check_center() {
        assert_eq!(127, to_raw(0.0));
    }

    #[test]
    fn check_merged() {
        let x = 37;
        let y = 185;
        let merged = to_merged(to_scaled(x), to_scaled(y));

        assert_eq!(x, (merged & 0xFF) as u8);
        assert_eq!(y, ((merged >> 8) & 0xFF) as u8);
    }

    #[test]
    fn check_xy_eq_dirlen() {
        let init = DVec2::new(0.1, 0.3);
        let dir = init.normalize();
        let len = init.length();
        assert_eq!(init, dir * len);
    }

    #[test]
    fn check_deadzone_scale() {
        let deadzone = 0.2;
        assert_eq!(0.0, deadzone_scale(0.0, deadzone));
        assert_eq!(0.0, deadzone_scale(0.2, deadzone));
        assert_eq!(1.0, deadzone_scale(1.0, deadzone));
        assert_eq!(0.375, deadzone_scale(0.5, deadzone));
    }
}
