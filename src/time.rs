//! Julian date conversion.
//!
//! Uses the Fliegel & Van Flandern (1968) algorithm for the Gregorian
//! calendar to Julian Day Number conversion, the same integer-arithmetic
//! formulation used by `sunrise` and `solar-positioning` — it avoids
//! loops entirely, which makes it straightforward for Kani to explore
//! exhaustively over a bounded input range.

/// A Julian date: days since noon UTC, 1 January 4713 BC (proleptic
/// Julian calendar).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct JulianDate(pub f64);

impl JulianDate {
    /// Builds a [`JulianDate`] from a Gregorian calendar date and
    /// fractional day.
    ///
    /// `year`/`month`/`day` follow the proleptic Gregorian calendar
    /// (astronomical year numbering: 1 BC is year 0, 2 BC is year -1).
    /// `day_fraction` is the fraction of the day elapsed since midnight
    /// UTC, in $[0, 1)$.
    ///
    /// Valid for `year` in `[-4800, 1_000_000]`; outside that range the
    /// result is still computed but not guaranteed free of overflow in
    /// `i64` arithmetic.
    pub fn from_gregorian(year: i32, month: u32, day: u32, day_fraction: f64) -> Self {
        let y = year as i64;
        let m = month as i64;
        let d = day as i64;

        // Fliegel & Van Flandern (1968), integer form.
        let jdn = (1461 * (y + 4800 + (m - 14) / 12)) / 4
            + (367 * (m - 2 - 12 * ((m - 14) / 12))) / 12
            - (3 * ((y + 4900 + (m - 14) / 12) / 100)) / 4
            + d
            - 32075;

        JulianDate(jdn as f64 - 0.5 + day_fraction)
    }

    /// Days elapsed since the J2000.0 epoch (2000-01-01T12:00:00 UTC,
    /// JD 2451545.0).
    #[inline]
    pub fn days_since_j2000(self) -> f64 {
        self.0 - 2451545.0
    }

    /// Julian centuries elapsed since J2000.0, the time unit most
    /// VSOP87/ELP2000-style series are parameterized in.
    #[inline]
    pub fn centuries_since_j2000(self) -> f64 {
        self.days_since_j2000() / 36525.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_epoch() {
        let jd = JulianDate::from_gregorian(2000, 1, 1, 0.5);
        assert!((jd.0 - 2451545.0).abs() < 1e-9);
        assert!(jd.days_since_j2000().abs() < 1e-9);
    }

    #[test]
    fn unix_epoch() {
        // 1970-01-01T00:00:00 UTC is JD 2440587.5.
        let jd = JulianDate::from_gregorian(1970, 1, 1, 0.0);
        assert!((jd.0 - 2440587.5).abs() < 1e-9);
    }

    #[test]
    fn known_date_vondrak() {
        // 2026-01-01T00:00:00 UTC: sanity-check against the day count,
        // not an external oracle, since this is a scaffolding test.
        let jd = JulianDate::from_gregorian(2026, 1, 1, 0.0);
        let prev = JulianDate::from_gregorian(2025, 1, 1, 0.0);
        // 2025 is not a leap year.
        assert!((jd.0 - prev.0 - 365.0).abs() < 1e-9);
    }
}

/// Kani proof harnesses. Run with `cargo kani --harness <name>`.
///
/// `from_gregorian`'s `i64` arithmetic has no loops, so Kani can check
/// it exhaustively over the full `i32`/`u32` domain of `year`/`month`/
/// `day` — no `kani::assume` range restriction needed, unlike the
/// `[-4800, 1_000_000]` doc comment, which only bounds where the
/// *calendar semantics* remain meaningful, not where the arithmetic is
/// overflow-safe.
#[cfg(kani)]
mod proofs {
    use super::*;

    /// `from_gregorian` never triggers an `i64` arithmetic overflow, for
    /// any `year: i32`, `month: u32`, `day: u32`.
    #[kani::proof]
    fn from_gregorian_never_overflows() {
        let year: i32 = kani::any();
        let month: u32 = kani::any();
        let day: u32 = kani::any();
        let day_fraction: f64 = kani::any();
        kani::assume(day_fraction.is_finite());
        let _ = JulianDate::from_gregorian(year, month, day, day_fraction);
    }

    /// `days_since_j2000`/`centuries_since_j2000` never panic, and
    /// preserve finiteness, for any finite `JulianDate`.
    #[kani::proof]
    fn epoch_conversions_never_panic() {
        let jd_value: f64 = kani::any();
        kani::assume(jd_value.is_finite());
        let jd = JulianDate(jd_value);
        assert!(jd.days_since_j2000().is_finite());
        assert!(jd.centuries_since_j2000().is_finite());
    }
}
