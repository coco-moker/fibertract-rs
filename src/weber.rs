//! Weber-law sensory quantization.
//!
//! Just-noticeable difference (JND) proportional to stimulus magnitude.
//! Small stimuli have fine resolution, large stimuli have coarse resolution.
//! This is how biology works: you notice 1g vs 2g but not 100g vs 101g.
//!
//! Step sizes follow biological sensory scaling:
//! - Low intensity (0-49): step 5 — fine discrimination
//! - Medium intensity (50-199): step 10 — moderate discrimination
//! - High intensity (200-999): step 15 — coarse discrimination
//! - Extreme intensity (1000+): step 25 — survival-only resolution

/// Quantize a raw sensory value using Weber-law step sizes.
///
/// Preserves sign. Zero passes through unchanged.
/// Output values snap to the nearest step boundary below magnitude.
///
/// # Examples
///
/// ```
/// use fibertract::weber_quantize;
///
/// assert_eq!(weber_quantize(23), 20);    // step=5, fine
/// assert_eq!(weber_quantize(130), 130);  // step=10, already on boundary
/// assert_eq!(weber_quantize(137), 130);  // step=10, snaps down
/// assert_eq!(weber_quantize(500), 495);  // step=15, coarse
/// assert_eq!(weber_quantize(-47), -45);  // preserves sign, step=5
/// assert_eq!(weber_quantize(0), 0);      // zero unchanged
/// ```
#[inline]
pub fn weber_quantize(raw: i32) -> i32 {
    if raw == 0 {
        return 0;
    }

    let abs_val = raw.unsigned_abs();

    let step: u32 = match abs_val {
        0..=49 => 5,
        50..=199 => 10,
        200..=999 => 15,
        _ => 25,
    };

    let quantized_abs = (abs_val / step) * step;

    if raw > 0 {
        quantized_abs as i32
    } else {
        -(quantized_abs as i32)
    }
}

/// Returns the Weber step size for a given magnitude.
///
/// Useful for determining discrimination resolution at a given intensity.
#[inline]
pub fn weber_step(magnitude: u32) -> u32 {
    match magnitude {
        0..=49 => 5,
        50..=199 => 10,
        200..=999 => 15,
        _ => 25,
    }
}

/// Returns the Weber fraction (JND / magnitude) as a u8 percentage.
///
/// Higher = coarser discrimination. Returns 255 for magnitude 0 (undefined).
#[inline]
pub fn weber_fraction_pct(magnitude: u32) -> u8 {
    if magnitude == 0 {
        return 255;
    }
    let step = weber_step(magnitude);
    let frac = step * 100 / magnitude;
    frac.min(255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_low_intensity() {
        assert_eq!(weber_quantize(0), 0);
        assert_eq!(weber_quantize(1), 0);
        assert_eq!(weber_quantize(4), 0);
        assert_eq!(weber_quantize(5), 5);
        assert_eq!(weber_quantize(7), 5);
        assert_eq!(weber_quantize(23), 20);
        assert_eq!(weber_quantize(49), 45);
    }

    #[test]
    fn quantize_medium_intensity() {
        assert_eq!(weber_quantize(50), 50);
        assert_eq!(weber_quantize(55), 50);
        assert_eq!(weber_quantize(130), 130);
        assert_eq!(weber_quantize(137), 130);
        assert_eq!(weber_quantize(199), 190);
    }

    #[test]
    fn quantize_high_intensity() {
        assert_eq!(weber_quantize(200), 195);
        assert_eq!(weber_quantize(500), 495);
        assert_eq!(weber_quantize(999), 990);
    }

    #[test]
    fn quantize_extreme_intensity() {
        assert_eq!(weber_quantize(1000), 1000);
        assert_eq!(weber_quantize(1024), 1000);
        assert_eq!(weber_quantize(5000), 5000);
        assert_eq!(weber_quantize(5013), 5000);
    }

    #[test]
    fn quantize_negative_preserves_sign() {
        assert_eq!(weber_quantize(-23), -20);
        assert_eq!(weber_quantize(-130), -130);
        assert_eq!(weber_quantize(-500), -495);
        assert_eq!(weber_quantize(-47), -45);
    }

    #[test]
    fn step_sizes() {
        assert_eq!(weber_step(0), 5);
        assert_eq!(weber_step(49), 5);
        assert_eq!(weber_step(50), 10);
        assert_eq!(weber_step(199), 10);
        assert_eq!(weber_step(200), 15);
        assert_eq!(weber_step(999), 15);
        assert_eq!(weber_step(1000), 25);
    }

    #[test]
    fn fraction_decreases_with_magnitude() {
        // Weber's law: fraction decreases as magnitude increases
        let f10 = weber_fraction_pct(10);   // 5/10 = 50%
        let f100 = weber_fraction_pct(100); // 10/100 = 10%
        let f500 = weber_fraction_pct(500); // 15/500 = 3%
        assert!(f10 > f100);
        assert!(f100 > f500);
    }
}
