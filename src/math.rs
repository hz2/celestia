//! Transcendental functions, routed to `std` or `libm` depending on
//! enabled features. This indirection is the only place the rest of the
//! crate touches a concrete `sin`/`cos` implementation.

#[inline]
pub fn sin(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sin()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::sin(x)
    }
}

#[inline]
pub fn cos(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.cos()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::cos(x)
    }
}

#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        y.atan2(x)
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::atan2(y, x)
    }
}

#[inline]
pub fn asin(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.asin()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::asin(x)
    }
}

#[inline]
pub fn sqrt(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sqrt()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::sqrt(x)
    }
}

#[inline]
pub fn floor(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::floor(x)
    }
}
