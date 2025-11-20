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
    pub const PI: Self = Self::from_2_longs(3, 1415926535897932384);

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

    /// Calculate factorial (n!)
    #[inline]
    pub const fn factorial(self) -> Self {
        assert!(self.0 >= 0, "Factorial of negative number");
        assert!(self.0 % Self::SCALE == 0, "Factorial of non-integer");

        Self(
            match self.0 as usize {
                0 => 1,
                1 => 1,
                2 => 2,
                3 => 6,
                4 => 24,
                5 => 120,
                6 => 720,
                7 => 5040,
                8 => 40320,
                9 => 362880,
                10 => 3628800,
                11 => 39916800,
                12 => 479001600,
                13 => 6227020800,
                14 => 87178291200,
                15 => 1307674368000,
                16 => 20922789888000,
                17 => 355687428096000,
                18 => 6402373705728000,
                19 => 121645100408832000,
                20 => 2432902008176640000i64,
                _ => panic!("Factorial will big what i64::MAX (n > 20)"),
            }
            .saturating_mul(Self::SCALE),
        )
    }

    /// Common Taylor series implementation
    #[inline]
    pub fn taylor_series(
        first_term: Self,
        mut next_term: impl FnMut(Self, usize) -> Self,
        max_iterations: Option<usize>,
    ) -> Self {
        let mut sum = first_term;
        let mut term = first_term;
        let mut n = 1;
        let max_iterations = max_iterations.unwrap_or(if F < 4 {
            6
        } else if F < 8 {
            10
        } else {
            15
        });

        while n < max_iterations && term.0.abs() > 10 {
            term = next_term(term, n);
            sum += term;
            n += 1;
        }

        sum
    }

    /// Normalize angle to [-π, π] range
    #[inline]
    pub fn normalize_angle(self) -> Self {
        let mut angle = self;

        // Remove full rotations (2π)
        if angle.0.abs() > Self::TAU.0 {
            let rotations = angle / Self::TAU;
            angle -= rotations * Self::TAU;
        }

        // Normalize to [-π, π]
        if angle > Self::PI {
            angle -= Self::TAU;
        } else if angle < -Self::PI {
            angle += Self::TAU;
        }

        angle
    }

    /// Calculate sine using Taylor series expansion
    #[inline]
    pub fn sin(self) -> Self {
        let mut x = self.normalize_angle();

        // For angles in [π/2, π] and [-π, -π/2], use sin(x) = sin(π - x)
        if x > Self::PI / Self::from_int(2) {
            x = Self::PI - x;
        } else if x < -Self::PI / Self::from_int(2) {
            x = -Self::PI - x;
        }

        let x2 = -x * x;

        Self::taylor_series(
            x,
            |term, n| {
                let n2 = (2 * n + 1) as i64;
                let divisor = Self::from_int(n2 * (2 * n as i64 + 2));
                term * x2 / divisor
            },
            None,
        )
    }

    /// Calculate cosine using identity cos(x) = sin(π/2 - x)
    #[inline]
    pub fn cos(self) -> Self {
        (Self::PI / Self::from_int(2) - self).sin()
    }

    /// Calculate tangent using identity tan(x) = sin(x) / cos(x)
    #[inline]
    pub fn tan(self) -> Self {
        let sin_val = self.sin();
        let cos_val = self.cos();

        // Handle division by zero for angles where cos(x) = 0
        if cos_val.0 == 0 {
            if sin_val.0 >= 0 {
                return Self::from_raw(i64::MAX);
            } else {
                return Self::from_raw(i64::MIN);
            }
        }

        sin_val / cos_val
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
