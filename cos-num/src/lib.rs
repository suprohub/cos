#![cfg_attr(not(feature = "std"), no_std)]

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
    pub const ONE: Self = Self::from_int(1);

    /// Archimedes' constant (π)
    pub const PI: Self = Self::from_2_longs(3, 1415926535897932384);

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    pub const TAU: Self = Self::from_2_longs(6, 2831853071795864769);

    /// The golden ratio (φ)
    pub const PHI: Self = Self::from_2_longs(1, 6180339887498948482);

    /// The Euler-Mascheroni constant (γ)
    pub const EGAMMA: Self = Self::from_2_longs(0, 5772156649015328606);

    /// Square root of 2 (√2)
    pub const SQRT_2: Self = Self::from_2_longs(1, 4142135623730950488);

    /// Euler's number (e)
    pub const E: Self = Self::from_2_longs(2, 7182818284590452353);

    /// Natural logarithm of 2 (ln(2))
    pub const LN_2: Self = Self::from_2_longs(0, 6931471805599453094);

    /// Create from raw inner representation (no scaling).
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    /// Get raw inner
    #[inline]
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Create from integer (integral value)
    #[inline]
    #[must_use]
    pub const fn from_int(n: i64) -> Self {
        Self(n.saturating_mul(Self::SCALE))
    }

    /// Create from f32 floating point value
    #[inline]
    #[must_use]
    pub const fn from_f32(value: f32) -> Self {
        // Handle special cases
        assert!(!value.is_nan(), "Cannot convert NaN to fixed-point number");

        if value.is_infinite() {
            if value.is_sign_positive() {
                return Self(i64::MAX);
            } else {
                return Self(i64::MIN);
            }
        }

        // Scale and round to nearest integer
        let scaled = value * (Self::SCALE as f32);

        // Handle overflow/underflow
        if scaled > i64::MAX as f32 {
            Self(i64::MAX)
        } else if scaled < i64::MIN as f32 {
            Self(i64::MIN)
        } else {
            Self(scaled.round() as i64)
        }
    }

    /// Create from f64 floating point value
    #[inline]
    #[must_use]
    pub const fn from_f64(value: f64) -> Self {
        // Handle special cases
        assert!(!value.is_nan(), "Cannot convert NaN to fixed-point number");

        if value.is_infinite() {
            if value.is_sign_positive() {
                return Self(i64::MAX);
            } else {
                return Self(i64::MIN);
            }
        }

        // Scale and round to nearest integer
        let scaled = value * (Self::SCALE as f64);

        // Handle overflow/underflow
        if scaled > i64::MAX as f64 {
            Self(i64::MAX)
        } else if scaled < i64::MIN as f64 {
            Self(i64::MIN)
        } else {
            Self(scaled.round() as i64)
        }
    }

    /// Create from integer and fraction
    #[inline]
    #[must_use]
    pub const fn from_2_longs(int: i64, frac: i64) -> Self {
        if F == 0 {
            Self(int)
        } else {
            // 19 safe - its maximum of i64
            Self(int.saturating_mul(Self::SCALE) + frac / 10i64.pow(19 - F as u32))
        }
    }

    #[inline]
    #[must_use]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Get square root of self
    #[must_use]
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
    #[must_use]
    pub const fn factorial(self) -> Self {
        assert!(self.0 >= 0, "Factorial of negative number");
        assert!(self.0 % Self::SCALE == 0, "Factorial of non-integer");

        Self(
            match self.0 / Self::SCALE {
                0 | 1 => 1,
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
            8
        } else if F < 8 {
            16
        } else {
            32
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn cos(self) -> Self {
        (Self::PI / Self::from_int(2) - self).sin()
    }

    /// Calculate tangent using identity tan(x) = sin(x) / cos(x)
    #[inline]
    #[must_use]
    pub fn tan(self) -> Self {
        self.sin() / self.cos()
    }

    /// Calculate cotangent using identity ctg(x) = cos(x) / sin(x)
    #[inline]
    #[must_use]
    pub fn ctg(self) -> Self {
        self.cos() / self.sin()
    }

    /// Calculate hyperbolic sine using Taylor series expansion
    #[inline]
    #[must_use]
    pub fn sinh(self) -> Self {
        let x = self;
        let x2 = x * x;

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

    /// Calculate hyperbolic cosine using Taylor series expansion
    #[inline]
    #[must_use]
    pub fn cosh(self) -> Self {
        let x = self;
        let x2 = x * x;

        Self::taylor_series(
            Self::ONE,
            |term, n| {
                let n2 = (2 * n) as i64;
                let divisor = Self::from_int(n2 * (2 * n as i64 - 1));
                term * x2 / divisor
            },
            None,
        )
    }

    /// Calculate hyperbolic tangent using identity tanh(x) = sinh(x) / cosh(x)
    #[inline]
    #[must_use]
    pub fn tanh(self) -> Self {
        self.sinh() / self.cosh()
    }

    /// Calculate hyperbolic cotangent using identity coth(x) = cosh(x) / sinh(x)
    #[inline]
    #[must_use]
    pub fn ctgh(self) -> Self {
        self.cosh() / self.sinh()
    }

    /// Calculate natural logarithm using Taylor series expansion
    ///
    /// Uses the identity: ln(x) = 2 * [ (x-1)/(x+1) + (1/3)*((x-1)/(x+1))^3 + (1/5)*((x-1)/(x+1))^5 + ... ]
    /// This series converges quickly for x close to 1.
    /// For values far from 1, uses reduction to range [0.5, 2] using identities.
    #[inline]
    #[must_use]
    pub fn ln(self) -> Self {
        assert!(self > Self::ZERO, "ln of non-positive number");
        if self == Self::ONE {
            return Self::ZERO;
        }

        let mut x = self;
        let mut result = Self::ZERO;

        // Reduce x to range [0.5, 2] using identities
        // For x > 2: ln(x) = ln(x/2) + ln(2)
        // For x < 0.5: ln(x) = ln(2x) - ln(2)
        while x > Self::from_int(2) {
            x /= Self::from_int(2);
            result += Self::LN_2;
        }

        while x < Self::from_f32(0.5) {
            // 0.5
            x *= Self::from_int(2);
            result -= Self::LN_2;
        }

        // Now x is in [0.5, 2], use Taylor series
        let y = (x - Self::ONE) / (x + Self::ONE);
        let y2 = y * y;

        Self::taylor_series(
            Self::from_int(2) * y,
            |term, n| {
                let coefficient = Self::from_int(2 * (2 * n as i64 + 3));
                term * y2 * coefficient / coefficient
            },
            Some(if F < 4 {
                8
            } else if F < 8 {
                16
            } else {
                32
            }),
        ) + result
    }

    /// Calculate area hyperbolic sine using logarithmic identity: arsinh(x) = ln(x + √(x² + 1))
    #[inline]
    #[must_use]
    pub fn arcsinh(self) -> Self {
        (self + (self * self + Self::ONE).sqrt()).ln()
    }

    /// Calculate area hyperbolic cosine using logarithmic identity: arcosh(x) = ln(x + √(x² - 1))
    #[inline]
    #[must_use]
    pub fn arccosh(self) -> Self {
        (self + (self * self - Self::ONE).sqrt()).ln()
    }

    /// Calculate area hyperbolic tangent using logarithmic identity: artanh(x) = 0.5 * ln((1 + x)/(1 - x))
    #[inline]
    #[must_use]
    pub fn arctanh(self) -> Self {
        ((Self::ONE + self) / (Self::ONE - self)).ln() / Self::from_int(2)
    }

    /// Calculate area hyperbolic cotangent using logarithmic identity: arcoth(x) = 0.5 * ln((x + 1)/(x - 1))
    #[inline]
    #[must_use]
    pub fn arcctgh(self) -> Self {
        ((self + Self::ONE) / (self - Self::ONE)).ln() / Self::from_int(2)
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

#[cfg(test)]
mod tests {
    use core::{f32, f64};

    use super::Num;

    // Test with 4 decimal places for good precision
    type TestNum = Num<4>;

    #[test]
    fn test_basic_operations() {
        // Addition
        assert_eq!(
            TestNum::from_int(2) + TestNum::from_int(3),
            TestNum::from_int(5)
        );
        assert_eq!(
            TestNum::from_f32(1.5) + TestNum::from_f32(2.25),
            TestNum::from_f32(3.75)
        );

        // Subtraction
        assert_eq!(
            TestNum::from_int(5) - TestNum::from_int(3),
            TestNum::from_int(2)
        );
        assert_eq!(
            TestNum::from_f32(4.5) - TestNum::from_f32(1.25),
            TestNum::from_f32(3.25)
        );

        // Multiplication
        assert_eq!(
            TestNum::from_int(3) * TestNum::from_int(4),
            TestNum::from_int(12)
        );
        assert_eq!(
            TestNum::from_f32(2.5) * TestNum::from_f32(4.0),
            TestNum::from_f32(10.0)
        );

        // Division
        assert_eq!(
            TestNum::from_int(10) / TestNum::from_int(2),
            TestNum::from_int(5)
        );
        assert_eq!(
            TestNum::from_f32(7.5) / TestNum::from_f32(2.5),
            TestNum::from_f32(3.0)
        );

        // Remainder
        assert_eq!(
            TestNum::from_int(7) % TestNum::from_int(3),
            TestNum::from_int(1)
        );
        assert_eq!(
            TestNum::from_f32(5.7) % TestNum::from_f32(2.2),
            TestNum::from_f32(1.3)
        );

        // Negation
        assert_eq!(-TestNum::from_int(5), TestNum::from_int(-5));
        assert_eq!(
            -TestNum::from_f32(f32::consts::PI),
            TestNum::from_f32(-f32::consts::PI)
        );

        // Absolute value
        assert_eq!(TestNum::from_int(-5).abs(), TestNum::from_int(5));
        assert_eq!(
            TestNum::from_f32(-f32::consts::PI).abs(),
            TestNum::from_f32(f32::consts::PI)
        );
    }

    #[test]
    fn test_assignment_operations() {
        let mut num = TestNum::from_int(10);

        num += TestNum::from_int(5);
        assert_eq!(num, TestNum::from_int(15));

        num -= TestNum::from_int(3);
        assert_eq!(num, TestNum::from_int(12));

        num *= TestNum::from_int(2);
        assert_eq!(num, TestNum::from_int(24));

        num /= TestNum::from_int(4);
        assert_eq!(num, TestNum::from_int(6));

        num %= TestNum::from_int(4);
        assert_eq!(num, TestNum::from_int(2));
    }

    #[test]
    fn test_comparisons() {
        // Equality
        assert_eq!(TestNum::from_int(5), TestNum::from_int(5));
        assert_eq!(
            TestNum::from_f32(f32::consts::PI),
            TestNum::from_f32(f32::consts::PI)
        );

        // Ordering
        assert!(TestNum::from_int(5) > TestNum::from_int(3));
        assert!(TestNum::from_int(3) < TestNum::from_int(5));
        assert!(TestNum::from_f32(2.5) >= TestNum::from_f32(2.5));
        assert!(TestNum::from_f32(1.8) <= TestNum::from_f32(1.8));
    }

    #[test]
    fn test_constructors() {
        // From raw
        assert_eq!(TestNum::from_raw(12345).raw(), 12345);

        // From integer
        assert_eq!(TestNum::from_int(42).raw(), 420000);

        // From f32
        assert_eq!(TestNum::from_f32(f32::consts::PI).raw(), 31416);

        // From f64
        assert_eq!(TestNum::from_f64(f64::consts::E).raw(), 27183);

        // From two longs
        assert_eq!(TestNum::from_2_longs(1, 2345000000000000000).raw(), 12345);
    }

    #[test]
    fn test_trigonometric_functions() {
        // Test sine function with common angles
        assert_eq!(TestNum::ZERO.sin(), TestNum::ZERO);
        assert_eq!(TestNum::PI.sin(), TestNum::ZERO);
        assert_eq!((TestNum::PI / TestNum::from_int(2)).sin(), TestNum::ONE);
        assert_eq!(
            (TestNum::PI / TestNum::from_int(6)).sin(),
            TestNum::from_f32(0.5)
        ); // 30°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(4)).sin(),
            TestNum::from_f32(f32::consts::FRAC_1_SQRT_2)
        ); // 45°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(3)).sin(),
            TestNum::from_f32(0.8660)
        ); // 60°

        // Test cosine function with common angles
        assert_eq!(TestNum::ZERO.cos(), TestNum::ONE);
        assert_eq!(TestNum::PI.cos(), -TestNum::ONE);
        assert_eq!((TestNum::PI / TestNum::from_int(2)).cos(), TestNum::ZERO);
        assert_eq!(
            (TestNum::PI / TestNum::from_int(3)).cos(),
            TestNum::from_f32(0.5)
        ); // 60°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(4)).cos(),
            TestNum::from_f32(f32::consts::FRAC_1_SQRT_2)
        ); // 45°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(6)).cos(),
            TestNum::from_f32(0.8660)
        ); // 30°

        // Test tangent function
        assert_eq!(TestNum::ZERO.tan(), TestNum::ZERO);
        assert_eq!((TestNum::PI / TestNum::from_int(4)).tan(), TestNum::ONE); // 45°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(6)).tan(),
            TestNum::from_f32(0.5774)
        ); // 30°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(3)).tan(),
            TestNum::from_f32(1.7321)
        ); // 60°

        // Test cotangent function
        assert_eq!((TestNum::PI / TestNum::from_int(4)).ctg(), TestNum::ONE); // 45°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(6)).ctg(),
            TestNum::from_f32(1.7321)
        ); // 30°
        assert_eq!(
            (TestNum::PI / TestNum::from_int(3)).ctg(),
            TestNum::from_f32(0.5774)
        ); // 60°

        // Test angle normalization
        let angle_2pi = TestNum::TAU + TestNum::PI / TestNum::from_int(4);
        assert_eq!(
            angle_2pi.normalize_angle(),
            TestNum::PI / TestNum::from_int(4)
        );

        let negative_angle = -TestNum::TAU - TestNum::PI / TestNum::from_int(4);
        assert_eq!(
            negative_angle.normalize_angle(),
            -TestNum::PI / TestNum::from_int(4)
        );

        let large_angle = TestNum::TAU * TestNum::from_int(3) + TestNum::PI / TestNum::from_int(3);
        assert_eq!(
            large_angle.normalize_angle(),
            TestNum::PI / TestNum::from_int(3)
        );
    }

    #[test]
    fn test_hyperbolic_functions() {
        // Test hyperbolic sine
        assert_eq!(TestNum::ZERO.sinh(), TestNum::ZERO);
        assert_eq!(TestNum::ONE.sinh(), TestNum::from_f32(1.1752));
        assert_eq!(TestNum::from_int(2).sinh(), TestNum::from_f32(3.6269));
        assert_eq!(TestNum::from_f32(-1.0).sinh(), TestNum::from_f32(-1.1752));

        // Test hyperbolic cosine
        assert_eq!(TestNum::ZERO.cosh(), TestNum::ONE);
        assert_eq!(TestNum::ONE.cosh(), TestNum::from_f32(1.5431));
        assert_eq!(TestNum::from_int(2).cosh(), TestNum::from_f32(3.7622));
        assert_eq!(TestNum::from_f32(-1.0).cosh(), TestNum::from_f32(1.5431)); // cosh is even function

        // Test hyperbolic tangent
        assert_eq!(TestNum::ZERO.tanh(), TestNum::ZERO);
        assert_eq!(TestNum::ONE.tanh(), TestNum::from_f32(0.7616));
        assert_eq!(TestNum::from_int(2).tanh(), TestNum::from_f32(0.9640));
        assert_eq!(TestNum::from_f32(-1.0).tanh(), TestNum::from_f32(-0.7616));

        // Test hyperbolic cotangent
        assert_eq!(TestNum::ONE.ctgh(), TestNum::from_f32(1.3130));
        assert_eq!(TestNum::from_int(2).ctgh(), TestNum::from_f32(1.0373));
        assert_eq!(TestNum::from_f32(-1.0).ctgh(), TestNum::from_f32(-1.3130));
    }

    #[test]
    fn test_logarithmic_functions() {
        // Test natural logarithm
        assert_eq!(TestNum::ONE.ln(), TestNum::ZERO);
        assert_eq!(TestNum::E.ln(), TestNum::ONE);
        assert_eq!(TestNum::from_int(2).ln(), TestNum::LN_2);
        assert_eq!(
            TestNum::from_int(10).ln(),
            TestNum::from_f32(f32::consts::LN_10)
        );
        assert_eq!(
            TestNum::from_f32(0.5).ln(),
            TestNum::from_f32(f32::consts::LN_2)
        );

        // Test inverse hyperbolic sine
        assert_eq!(TestNum::ZERO.arcsinh(), TestNum::ZERO);
        assert_eq!(TestNum::ONE.arcsinh(), TestNum::from_f32(0.8814));
        assert_eq!(TestNum::from_int(2).arcsinh(), TestNum::from_f32(1.4436));

        // Test inverse hyperbolic cosine
        assert_eq!(TestNum::ONE.arccosh(), TestNum::ZERO);
        assert_eq!(TestNum::from_int(2).arccosh(), TestNum::from_f32(1.3170));
        assert_eq!(TestNum::from_int(3).arccosh(), TestNum::from_f32(1.7627));

        // Test inverse hyperbolic tangent
        assert_eq!(TestNum::ZERO.arctanh(), TestNum::ZERO);
        assert_eq!(TestNum::from_f32(0.5).arctanh(), TestNum::from_f32(0.5493));
        assert_eq!(
            TestNum::from_f32(-0.5).arctanh(),
            TestNum::from_f32(-0.5493)
        );

        // Test inverse hyperbolic cotangent
        assert_eq!(TestNum::from_int(2).arcctgh(), TestNum::from_f32(0.5493));
        assert_eq!(TestNum::from_int(3).arcctgh(), TestNum::from_f32(0.3466));
        assert_eq!(TestNum::from_int(-2).arcctgh(), TestNum::from_f32(-0.5493));
    }

    #[test]
    fn test_other_mathematical_functions() {
        // Test square root with perfect squares
        assert_eq!(TestNum::ZERO.sqrt(), TestNum::ZERO);
        assert_eq!(TestNum::ONE.sqrt(), TestNum::ONE);
        assert_eq!(TestNum::from_int(4).sqrt(), TestNum::from_int(2));
        assert_eq!(TestNum::from_int(9).sqrt(), TestNum::from_int(3));
        assert_eq!(TestNum::from_int(16).sqrt(), TestNum::from_int(4));
        assert_eq!(TestNum::from_int(25).sqrt(), TestNum::from_int(5));

        // Test square root with non-perfect squares
        assert_eq!(TestNum::from_int(2).sqrt(), TestNum::SQRT_2);
        assert_eq!(TestNum::from_int(3).sqrt(), TestNum::from_f32(1.7321));
        assert_eq!(TestNum::from_int(5).sqrt(), TestNum::from_f32(2.2361));
        assert_eq!(TestNum::from_f32(0.25).sqrt(), TestNum::from_f32(0.5));
        assert_eq!(TestNum::from_f32(1.44).sqrt(), TestNum::from_f32(1.2));

        // Test factorial
        assert_eq!(TestNum::ZERO.factorial(), TestNum::ONE);
        assert_eq!(TestNum::ONE.factorial(), TestNum::ONE);
        assert_eq!(TestNum::from_int(2).factorial(), TestNum::from_int(2));
        assert_eq!(TestNum::from_int(3).factorial(), TestNum::from_int(6));
        assert_eq!(TestNum::from_int(4).factorial(), TestNum::from_int(24));
        assert_eq!(TestNum::from_int(5).factorial(), TestNum::from_int(120));
        assert_eq!(TestNum::from_int(6).factorial(), TestNum::from_int(720));
    }

    #[test]
    fn test_deref_and_borrow() {
        let mut num = TestNum::from_int(42);

        // Test deref
        assert_eq!(*num, 420000);

        // Test deref mut
        *num += 1000;
        assert_eq!(*num, 421000);

        // Test as_ref and as_mut
        assert_eq!(*num.as_ref(), 421000);
        *num.as_mut() += 1000;
        assert_eq!(*num, 422000);

        // Test borrow
        /*let borrowed: &i64 = num.borrow();
        assert_eq!(*borrowed, 422000);

        // Test borrow_mut
        *num.borrow_mut() += 1000;
        assert_eq!(*num, 423000);*/
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_division_by_zero() {
        let _ = TestNum::from_int(1) / TestNum::ZERO;
    }

    #[test]
    #[should_panic(expected = "sqrt of negative number")]
    fn test_sqrt_negative() {
        let _ = TestNum::from_int(-1).sqrt();
    }

    #[test]
    #[should_panic(expected = "Factorial of negative number")]
    fn test_factorial_negative() {
        let _ = TestNum::from_int(-1).factorial();
    }

    #[test]
    #[should_panic(expected = "ln of non-positive number")]
    fn test_ln_non_positive() {
        let _ = TestNum::ZERO.ln();
    }

    #[test]
    fn test_taylor_series_helper() {
        // Test that taylor_series works for a simple geometric series
        let result = TestNum::taylor_series(
            TestNum::ONE,
            |term, _n| term / TestNum::from_int(2),
            Some(5),
        );

        // 1 + 1/2 + 1/4 + 1/8 + 1/16 = 1.9375
        let expected = TestNum::from_f32(1.9375);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_different_scales() {
        // Test with zero fractional digits
        type IntegerNum = Num<0>;
        assert_eq!(
            IntegerNum::from_int(5) + IntegerNum::from_int(3),
            IntegerNum::from_int(8)
        );
        assert_eq!(
            IntegerNum::from_int(10) / IntegerNum::from_int(3),
            IntegerNum::from_int(3)
        ); // Integer division

        // Test with more fractional digits
        type HighPrecisionNum = Num<8>;
        assert_eq!(
            HighPrecisionNum::from_f32(1.5) + HighPrecisionNum::from_f32(2.25),
            HighPrecisionNum::from_f32(3.75)
        );
        assert_eq!(
            HighPrecisionNum::from_int(1).sqrt(),
            HighPrecisionNum::from_int(1)
        );
    }
}
