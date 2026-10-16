//! Angle normalization.
//!
//! Normalizing an angle to $[0, 2\pi)$ by repeated subtraction is
//! unbounded for pathological inputs (e.g. `f64::MAX`), which is both
//! slow and not Kani-provable without an input bound. [`normalize`]
//! instead uses a single division + floor, which is the pattern
//! `sunrise` and `solar-positioning` both use for the same reason.

use crate::math;
use core::f64::consts::TAU;

/// Normalizes `radians` to the half-open interval $[0, 2\pi)$.
///
/// # Examples
///
/// ```
/// use celestia::angle::normalize;
/// use core::f64::consts::PI;
///
/// assert!((normalize(3.0 * PI) - PI).abs() < 1e-12);
/// assert!((normalize(-PI) - PI).abs() < 1e-12);
/// ```
#[inline]
pub fn normalize(radians: f64) -> f64 {
    let wrapped = radians - TAU * math::floor(radians / TAU);
    // `floor(radians / TAU)` can round to one more than the
    // mathematically exact quotient near a multiple of TAU, pushing
    // `wrapped` just past either edge of [0, TAU); clamp both edges.
    if wrapped < 0.0 || wrapped >= TAU {
        0.0
    } else {
        wrapped
    }
}

/// Converts degrees to radians.
#[inline]
pub fn to_radians(degrees: f64) -> f64 {
    degrees * (core::f64::consts::PI / 180.0)
}

/// Converts radians to degrees.
#[inline]
pub fn to_degrees(radians: f64) -> f64 {
    radians * (180.0 / core::f64::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_in_range() {
        for i in -20..20 {
            let x = i as f64 * 1.3;
            let n = normalize(x);
            assert!((0.0..TAU).contains(&n), "normalize({x}) = {n}");
        }
    }

    #[test]
    fn normalize_preserves_angle_mod_tau() {
        let x = 7.0 * TAU + 1.234;
        assert!((normalize(x) - 1.234).abs() < 1e-9);
    }
}

/// Kani proof harnesses. Run with `cargo kani --harness <name>`.
///
/// `normalize` only uses `floor`, `*`, `-`, and a comparison -- no loops,
/// no `sin`/`cos`/`sqrt` (the operations Kani over-approximates, per
/// <https://model-checking.github.io/kani/rust-feature-support.html#floating-point-operations>)
/// -- so this is in Kani's fully-precise, non-approximated subset.
#[cfg(kani)]
mod proofs {
    use super::*;

    /// `normalize` never panics for any finite `f64` input.
    ///
    /// Excludes NaN/infinity: `radians / TAU` is `NaN` or `inf` for
    /// those, and the subsequent subtraction legitimately produces
    /// `NaN` per IEEE 754 (`inf - inf = NaN`) -- that's correct
    /// floating-point semantics, not a panic, but it does trip Kani's
    /// default NaN-production check, which is a stricter property than
    /// "never panics".
    #[kani::proof]
    fn normalize_never_panics() {
        let x: f64 = kani::any();
        kani::assume(x.is_finite());
        let _ = normalize(x);
    }

    /// For any finite input, `normalize` returns a value in $[0, 2\pi)$.
    #[kani::proof]
    fn normalize_result_in_range() {
        let x: f64 = kani::any();
        kani::assume(x.is_finite());
        let n = normalize(x);
        assert!(n.is_finite());
        assert!(n >= 0.0);
        assert!(n < TAU);
    }

    /// `to_radians`/`to_degrees` never panic for any finite input.
    #[kani::proof]
    fn conversions_never_panic() {
        let d: f64 = kani::any();
        kani::assume(d.is_finite());
        let r = to_radians(d);
        assert!(r.is_finite() || !d.is_finite());
        let _ = to_degrees(r);
    }
}
