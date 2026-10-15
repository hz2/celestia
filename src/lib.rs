//! `celestia` — `no_std` sun and moon position/phase calculations.
//!
//! Implements truncated analytic series in the style of VSOP87 (planets)
//! and ELP2000-82B (Moon), following the truncation strategy used by
//! `cosinekitty/astronomy` and `pracstro`: keep only the terms needed to
//! stay within a stated accuracy budget (on the order of $1' = 2.9
//! \times 10^{-4}$ rad), rather than shipping full-precision series or
//! JPL binary ephemeris kernels.
//!
//! A proved bound on the floating-point rounding error of the series
//! evaluation itself (`gappa/` in the repository) shows that rounding
//! error is many orders of magnitude below the series truncation error,
//! for both correctly-rounded and typical 1-ulp trigonometric
//! implementations — so the accuracy budget is set by term count, not
//! arithmetic.
//!
//! # Features
//!
//! - `std` (default): native `f64` transcendental functions.
//! - `libm`: pure-Rust `libm` for `no_std` targets.
//!
//! Exactly one of `std`/`libm` must be enabled.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error!("either the `std` or `libm` feature is required");

pub mod angle;
pub mod math;
pub mod series;
pub mod time;
