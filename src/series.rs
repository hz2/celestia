//! Truncated trigonometric series evaluation.
//!
//! A single coordinate correction in a VSOP87/ELP2000-style theory is a
//! sum of periodic terms:
//!
//! $$ s = \sum_{i=1}^{n} a_i \cos(x_i) $$
//!
//! (or $\sin$ for terms in quadrature). [`Term`] stores one $(a_i,
//! x_i)$ pair; [`evaluate_cos`]/[`evaluate_sin`] fold a slice of terms
//! into $s$ via sequential `f64` summation.
//!
//! `gappa/series_sum_caseA.g` and `series_sum_caseB.g` prove that for a
//! representative 6-term series, this evaluation's rounding error
//! relative to the exact real-valued sum is bounded by $2^{-53.98}$
//! rad (correctly-rounded cosine) or $2^{-53.67}$ rad (typical 1-ulp
//! libm cosine) -- both far below the $\sim 2.9 \times 10^{-4}$ rad
//! (1 arcminute) truncation error a series of this length carries by
//! construction. Rounding error is not the accuracy bottleneck for this
//! class of algorithm; term count is.

use crate::math;

/// One term of a truncated trigonometric series: amplitude `a` (same
/// unit as the series result, typically radians) and argument `x`
/// (radians), evaluated as `a * cos(x)` or `a * sin(x)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Term {
    pub amplitude: f64,
    pub argument: f64,
}

impl Term {
    #[inline]
    pub const fn new(amplitude: f64, argument: f64) -> Self {
        Term {
            amplitude,
            argument,
        }
    }
}

/// The six amplitudes `gappa/series_sum_caseA.g` and `series_sum_caseB.g`
/// prove a rounding-error bound for. Kept as the single source of truth
/// so the Rust-side test/bench series can't silently drift from what the
/// proof actually covers; the `.g` scripts duplicate these as literals
/// since Gappa has no way to import a Rust constant -- if these change,
/// the `.g` files need the matching edit.
pub const GAPPA_PROOF_AMPLITUDES: [f64; 6] =
    [1.0975e-1, 1.274e-2, 6.58e-4, 2.14e-4, 1.11e-4, 4.6e-5];

/// Evaluates $\sum_i a_i \cos(x_i)$ over `terms`.
#[inline]
pub fn evaluate_cos(terms: &[Term]) -> f64 {
    terms
        .iter()
        .fold(0.0, |acc, t| acc + t.amplitude * math::cos(t.argument))
}

/// Evaluates $\sum_i a_i \sin(x_i)$ over `terms`.
#[inline]
pub fn evaluate_sin(terms: &[Term]) -> f64 {
    terms
        .iter()
        .fold(0.0, |acc, t| acc + t.amplitude * math::sin(t.argument))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_series_is_zero() {
        assert_eq!(evaluate_cos(&[]), 0.0);
        assert_eq!(evaluate_sin(&[]), 0.0);
    }

    #[test]
    fn single_term_cos() {
        let terms = [Term::new(2.0, 0.0)];
        assert!((evaluate_cos(&terms) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn single_term_sin_quarter_turn() {
        let terms = [Term::new(3.0, core::f64::consts::FRAC_PI_2)];
        assert!((evaluate_sin(&terms) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn matches_gappa_proof_series() {
        // evaluated at arbitrary arguments; this just checks the fold
        // order matches what was modeled in the proof (left-to-right
        // accumulation).
        let arguments = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let terms: Vec<Term> = GAPPA_PROOF_AMPLITUDES
            .iter()
            .zip(arguments)
            .map(|(&a, x)| Term::new(a, x))
            .collect();
        let direct: f64 = terms.iter().map(|t| t.amplitude * t.argument.cos()).sum();
        assert!((evaluate_cos(&terms) - direct).abs() < 1e-15);
    }
}

/// Kani proof harnesses. Run with `cargo kani --harness <name>`.
///
/// `cos`/`sin` themselves are over-approximated by Kani (any value in
/// $[-1, 1]$ regardless of input, per
/// <https://model-checking.github.io/kani/rust-feature-support.html#floating-point-operations>),
/// so these harnesses can only prove panic-freedom / finiteness of the
/// *fold*, not the accuracy claims -- that's what `gappa/` proves
/// instead, on the arithmetic in isolation from the trig calls.
///
/// The fold has a loop whose bound equals `terms.len()`; Kani needs that
/// bound fixed and small to unwind it, so these harnesses use a 6-term
/// array (matching the Gappa proof's series length) with
/// `#[kani::unwind(7)]` (one more than the term count, per the Kani
/// tutorial's off-by-one guidance).
#[cfg(kani)]
mod proofs {
    use super::*;

    fn arbitrary_terms() -> [Term; 6] {
        let mut terms = [Term::new(0.0, 0.0); 6];
        for t in &mut terms {
            let amplitude: f64 = kani::any();
            let argument: f64 = kani::any();
            kani::assume(amplitude.is_finite());
            kani::assume(argument.is_finite());
            *t = Term::new(amplitude, argument);
        }
        terms
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn evaluate_cos_never_panics() {
        let terms = arbitrary_terms();
        let _ = evaluate_cos(&terms);
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn evaluate_sin_never_panics() {
        let terms = arbitrary_terms();
        let _ = evaluate_sin(&terms);
    }

    /// With bounded (non-huge) amplitudes, the fold result stays finite
    /// -- i.e. no term's `amplitude * cos(x)` can overflow to infinity
    /// and poison the running sum, given `cos`/`sin`'s output range is
    /// `[-1, 1]` even under Kani's over-approximation.
    #[kani::proof]
    #[kani::unwind(7)]
    fn evaluate_cos_finite_for_bounded_amplitudes() {
        let mut terms = [Term::new(0.0, 0.0); 6];
        for t in &mut terms {
            let amplitude: f64 = kani::any();
            let argument: f64 = kani::any();
            kani::assume(amplitude.abs() < 1e100);
            kani::assume(argument.is_finite());
            *t = Term::new(amplitude, argument);
        }
        assert!(evaluate_cos(&terms).is_finite());
    }
}
