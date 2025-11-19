use core::{
    borrow::{Borrow, BorrowMut},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
        SubAssign,
    },
};
use ufmt::derive::uDebug;

/// Fixed-point numeric type with compile-time decimal scaling.
///
/// Num stores a signed 64-bit integer that represents a fixed-point value
/// with F decimal fractional digits. The underlying stored value is the
/// integer representation scaled by 10^F. For example, `Num::<2>::from_int(3)`
/// stores 300 and represents 3.00.
#[derive(Debug, uDebug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Num<const F: u8>(pub i64);

impl<const F: u8> Num<F> {
    /// Current scale of frac
    pub const SCALE: i64 = {
        let mut s: i64 = 1;
        let mut i = 0u8;
        while i < F {
            s *= 10;
            i += 1;
        }
        s
    };

    /// Just a 0 incapsulated in `Num`
    pub const ZERO: Self = Self(0);

    /// Just a 1 incapsulated in `Num`
    pub const ONE: Self = Self(1);

    /// Archimedes' constant (π)
    pub const PI: Self = Self::from_2_longs(3i64, 1415926535897932384i64);

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    pub const TAU: Self = Self::from_2_longs(6, 2831853071795864769);

    /// The golden ratio (φ)
    pub const PHI: Self = Self::from_2_longs(6, 2831853071795864769);

    /// The Euler-Mascheroni constant (γ)
    pub const EGAMMA: Self = Self::from_2_longs(0, 5772156649015328606);

    /// Square root of 2 (√2)
    pub const SQRT_2: Self = Self::from_2_longs(1, 4142135623730950488);

    /// Euler's number (e)
    pub const E: Self = Self::from_2_longs(2, 7182818284590452353);

    /// Create from raw inner representation (no scaling).
    #[inline]
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    /// Get raw inner
    #[inline]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Create from integer (integral value)
    #[inline]
    pub const fn from_int(n: i64) -> Self {
        Self(n.saturating_mul(Self::SCALE))
    }

    /// Create from integer and fraction
    #[inline]
    pub const fn from_2_longs(int: i64, frac: i64) -> Self {
        if F == 0 {
            Self(int)
        } else {
            Self(int.saturating_mul(Self::SCALE) + frac / 10i64.pow(19 - F as u32))
        }
    }

    /// Get square root of self
    pub const fn sqrt(self) -> Self {
        // Why i dont use `Self(self.0.wrapping_mul(Self::SCALE).isqrt())`?
        // Cool question, because my code looks weird like why
        // if we already have 0i32.isqrt(). So, i have answer:
        // Rust isqrt impl: 12754 bytes to flash
        // My isqrt impl: 11344 bytes to flash
        // Idk why this happen

        assert!(self.0 >= 0, "sqrt of negative number");

        if self.0 == 0 {
            return Self::ZERO;
        }

        let n = self.0 * Self::SCALE;
        let mut x0 = n;
        let mut x1 = i64::midpoint(x0, n / x0);

        while x1 < x0 {
            x0 = x1;
            x1 = i64::midpoint(x0, n / x0);
        }

        // Round
        let diff = n - x0 * x0;
        if diff * 2 < 2 * x0 + 1 {
            Self(x0)
        } else {
            Self(x0 + 1)
        }
    }
}

impl<const F: u8> Add for Num<F> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl<const F: u8> Sub for Num<F> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl<const F: u8> Neg for Num<F> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self(self.0.wrapping_neg())
    }
}

impl<const F: u8> Mul for Num<F> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // Compute (a * b) / S with rounding to nearest
        let r = self.0.wrapping_mul(rhs.0);

        // Add half of the scale factor for rounding
        let rounded = if r >= 0 {
            (r + Self::SCALE / 2) / Self::SCALE
        } else {
            (r - Self::SCALE / 2) / Self::SCALE
        };

        Self(rounded)
    }
}

impl<const F: u8> Div for Num<F> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        // Panic on zero
        // Idk why but this make program size smaller
        assert!(rhs.0 != 0, "division by zero");

        let r = self.0.wrapping_mul(Self::SCALE);

        // Add half of the divisor for rounding
        let rounded = if r >= 0 {
            (r + rhs.0 / 2) / rhs.0
        } else {
            (r - rhs.0 / 2) / rhs.0
        };

        Self(rounded)
    }
}

impl<const F: u8> Rem for Num<F> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}

impl<const F: u8> AddAssign for Num<F> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const F: u8> SubAssign for Num<F> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<const F: u8> MulAssign for Num<F> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<const F: u8> DivAssign for Num<F> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<const F: u8> RemAssign for Num<F> {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl<const F: u8> AsRef<i64> for Num<F> {
    #[inline]
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl<const F: u8> AsMut<i64> for Num<F> {
    #[inline]
    fn as_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}

impl<const F: u8> Borrow<i64> for Num<F> {
    #[inline]
    fn borrow(&self) -> &i64 {
        &self.0
    }
}

impl<const F: u8> BorrowMut<i64> for Num<F> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}

impl<const F: u8> Deref for Num<F> {
    type Target = i64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const F: u8> DerefMut for Num<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
