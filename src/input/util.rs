const RAW_CENTER: f64 = 127.5;

/// Square (n * n)
#[inline]
pub const fn sq(n: f64) -> f64 {
    n * n
}

/// Converts raw value into -1.0 to 1.0 range
pub const fn to_scaled(raw: u8) -> f64 {
    (RAW_CENTER - raw as f64) * (1.0 / RAW_CENTER)
}

/// Converts scaled value into raw value
pub fn to_raw(scaled: f64) -> u8 {
    255 - (RAW_CENTER + (scaled * RAW_CENTER)).round() as u8
}

/// Creates a single u16 containing 2 raw values
pub fn to_merged(x: f64, y: f64) -> u16 {
    let x = to_raw(x) as u16;
    let y = to_raw(y) as u16;
    (x & 0x00FF) + ((y << 8) & 0xFF00)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_conversion() {
        for x in 0..=255 {
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
