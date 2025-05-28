/// Square (n * n)
#[inline]
pub const fn sq(n: f64) -> f64 {
    n * n
}

/// Scale length outside of deadzone to full 0.0 to 1.0+ range
pub const fn deadzone_scale(len: f64, deadzone: f64) -> f64 {
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
        assert_eq!(0.375, deadzone_scale(0.5, DEADZONE));
        assert_eq!(1.000, deadzone_scale(1.0, DEADZONE));
    }
}
