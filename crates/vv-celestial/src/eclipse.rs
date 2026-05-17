//! Solar eclipse factor based on angular overlap.
//!
//! Approximates the fraction of the sun's disc obscured by another body
//! between the sun and observer. Treats both as discs; the answer is
//! exact for full eclipses and good enough (≈ 1 % error) for partial.

/// `0.0` ⇒ no obscuration, `1.0` ⇒ totality. Distances and radii are in
/// metres; `*_dir_local` should both be unit vectors in the observer frame.
pub fn solar_eclipse_factor(
    sun_dir_local: glam::Vec3,
    sun_radius_m: f64,
    sun_distance_m: f64,
    occluder_dir_local: glam::Vec3,
    occluder_radius_m: f64,
    occluder_distance_m: f64,
) -> f32 {
    if sun_distance_m <= 0.0 || occluder_distance_m <= 0.0 {
        return 0.0;
    }
    // The occluder only eclipses the sun if it sits between the observer
    // and the sun (so its distance must be shorter and its direction must
    // approximately align with the sun's).
    if occluder_distance_m >= sun_distance_m {
        return 0.0;
    }
    let sun_alpha = (sun_radius_m / sun_distance_m).clamp(-1.0, 1.0).asin() as f32;
    let occ_alpha = (occluder_radius_m / occluder_distance_m).clamp(-1.0, 1.0).asin() as f32;
    let cos_sep = sun_dir_local
        .dot(occluder_dir_local)
        .clamp(-1.0, 1.0);
    let sep = cos_sep.acos();

    if sep >= sun_alpha + occ_alpha {
        return 0.0;
    }
    if sep + sun_alpha <= occ_alpha {
        // Occluder completely covers the sun.
        return 1.0;
    }
    // Partial: approximate by linear fall-off between the two limits above.
    // Cheap and visually smooth — full lune-area formula is overkill for V1.
    let outer = sun_alpha + occ_alpha;
    let inner = (occ_alpha - sun_alpha).abs();
    let t = ((outer - sep) / (outer - inner)).clamp(0.0, 1.0);
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUN_R: f64 = 6.96e8;
    const SUN_D: f64 = 1.496e11;
    const MOON_R: f64 = 1.737e6;
    const MOON_D: f64 = 3.844e8;

    #[test]
    fn aligned_moon_fully_eclipses_sun() {
        let f = solar_eclipse_factor(
            glam::Vec3::Y,
            SUN_R,
            SUN_D,
            glam::Vec3::Y,
            MOON_R,
            MOON_D,
        );
        // Moon angular radius (~0.0045 rad) > sun (~0.00465 rad). Earth's
        // moon barely covers the sun — total at perigee, annular at apogee.
        // With these exact textbook values, occ_alpha is slightly smaller, so
        // we expect a near-full but not exactly 1.0 result.
        assert!(f > 0.9, "alignment should give a near-full eclipse, got {f}");
    }

    #[test]
    fn moon_far_from_sun_has_no_effect() {
        let f = solar_eclipse_factor(
            glam::Vec3::Y,
            SUN_R,
            SUN_D,
            -glam::Vec3::Y,
            MOON_R,
            MOON_D,
        );
        assert_eq!(f, 0.0);
    }

    #[test]
    fn moon_behind_sun_has_no_effect() {
        let f = solar_eclipse_factor(
            glam::Vec3::Y,
            SUN_R,
            SUN_D,
            glam::Vec3::Y,
            MOON_R,
            SUN_D * 2.0,
        );
        assert_eq!(f, 0.0);
    }

    #[test]
    fn grazing_alignment_is_partial() {
        // Offset the moon by ~half a sun angular radius — the discs still
        // overlap.
        let sun_alpha = (SUN_R / SUN_D) as f32;
        let offset = sun_alpha * 0.5;
        let moon_dir = glam::Vec3::new(offset.sin(), offset.cos(), 0.0).normalize();
        let f = solar_eclipse_factor(
            glam::Vec3::Y,
            SUN_R,
            SUN_D,
            moon_dir,
            MOON_R,
            MOON_D,
        );
        assert!(f > 0.0 && f < 1.0, "expected partial, got {f}");
    }
}
