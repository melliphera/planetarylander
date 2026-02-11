//! Integer-wrapping type intended to be used as a float replacement.
//! Two differently scaled versions of this are defined in the project;
//! 1) 56 fractional bits, used for quaternions and unit vectors.
//! 2) 7 fractional bits, used for anything that deals with distance.
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub(crate) const UNIT_FIXED_POINT_DECIMAL_BITS: u8 = 60;
pub(crate) const STEP_FIXED_POINT_DECIMAL_BITS: u8 = 40;
pub(crate) const SOLAR_FIXED_POINT_DECIMAL_BITS: u8 = 6; // bounded by Jupiter GM.

pub type UnitFp = FixedPoint<UNIT_FIXED_POINT_DECIMAL_BITS>;
pub type StepFp = FixedPoint<STEP_FIXED_POINT_DECIMAL_BITS>;
pub type SolarFp = FixedPoint<SOLAR_FIXED_POINT_DECIMAL_BITS>;

// const value of N means internal i64 representing 1 is 1 << N;
// 56 sub unit bits means 10^-17 precision
// 1 sign bit
// 7 bits left for integers, meaning a value range of +/- 128
// this leaves a very convenient interface for unit tests in the form of from_int().
#[must_use] // avoids "i think this mutates the input" errors.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint<const N: u8>(pub(crate) i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatConversionError {
    NonNumericInput,
    OutOfBounds,
}

impl<const N: u8> FixedPoint<N> {
    pub fn from_int(int: i64) -> Self {
        // creates a FixedPoint which REPRESENTS the selected int. To create one with a specified stored int, use with_internal().
        Self(int << N)
    }

    pub fn with_internal(int: i64) -> Self {
        // Instantiates a FixedPoint with the specified internal value.
        Self(int)
    }

    pub const fn from_f64(float: f64) -> Result<Self, FloatConversionError> {
        // handle trivial cases - subnormal is *significantly* below the finest precision.
        if float == 0.0 || float.is_subnormal() {
            return Ok(Self(0));
        };

        // reject any non-numeric f64s
        if !float.is_finite() {
            return Err(FloatConversionError::NonNumericInput);
        }

        // bounds check the exponent
        let mut exponent = ((float.to_bits() >> 52) & 0x7FF) as i64; // (MR A.2a) 11 bits, biased. Number must be 0 < 2047 as per IEEE 754
        exponent -= 1023; // unbiased
        if 63 - exponent <= N as i64 {
            // (MR A.2a) u8 is a valid subset of i64.
            return Err(FloatConversionError::OutOfBounds);
        }

        Ok(Self::from_f64_trusted(float))
    }

    pub const fn from_f64_trusted(float: f64) -> Self {
        //! ONLY TO BE USED WITH EXPLICITLY VALID MAGIC NUMBERS.
        //! Assumes a valid input, avoids bounds checking and results as a result.
        if float == 0.0 || float.is_subnormal() {
            return Self(0);
        };

        // isolate the sign, exponent and mantissa of the float in raw bits
        let bits = float.to_bits();
        let sign = (bits >> 63) as i64; // (MR A.2a) 0 or 1
        let mut exponent = ((bits >> 52) & 0x7FF) as i64; // (MR A.2a) 11 bits, biased. Number must be 0 < 2047 as per IEEE 754
        let mut mantissa = (bits & 0xFFFFFFFFFFFFF) as i128; // (MR A.2a) 52 bits. Comfortably fits in 128-bit register.

        // process these values numerically
        let signum = if sign == 0 { 1 } else { -1 }; // from sign bit to a number we can mutliply by.
        exponent -= 1023; // unbias the exponent;
        mantissa += 1 << 52; // add the implicit 1 from IEEE754 explicitly.

        // now float_value = signum * 2^exponent * mantissa
        // manipulate exponent and mantissa while holding the above true
        // when exponent is 0, shift should be such that the 52nd bit (0-ord) shifts to DECIMAL_BITS(0-ord)
        let shift = exponent + N as i64 - 52; // (MR A.2a) u8 is a valid subset of i64.
        if shift >= 0 {
            mantissa <<= shift;
        } else {
            mantissa >>= -shift;
        };

        // after this shift, mantissa represents the internal int of SubDecimal
        Self(signum * mantissa as i64) // (MR A.2b) Pre-shift, mantissa has 53 bits. For this to fail, shift must be >11. (multi-line justification)
                                       // therefore exponent + N must be > 63. This would put float out of bounds, which would be picked up by the bounds checks in from_f64().
                                       // this function is only called either by from_f64 or directly in test cases only. Therefore that bound is sufficient.
    }

    pub fn to_f64(&self) -> f64 {
        // produces the f64 representation of the represented number.
        if self.0 == 0 {
            return 0.0f64;
        }

        // extract sign bit, then convert to unsigned to work with value bits only.
        let sign_bit_placed: u64 = if self.0.signum() == 1 { 0 } else { 1 << 63 };
        let unsigned = self.0.unsigned_abs();

        // extract exponent bits
        let l0 = unsigned.leading_zeros() as u64; // (MR A.2a) u32->u64, direct superset.
        let internal_exponent: u64 = 63 - l0; // get the raw exponent value of the internal number.
        let exp_value = internal_exponent + 1023 - N as u64; // apply the exponent bias; subtract the const generic bias. Done this way to avoid underflow.
        let exp_bits_placed = exp_value << 52; // shift the exponent bits into their place.

        // calculate IEEE 754 (implicit-lead 52-bit) mantissa
        // first, shift values to represent explicit-lead 53-bit mantissa
        let explicit_lead_mantissa: u64 = if l0 < 11 {
            unsigned >> (11 - l0)
        } else if l0 == 11 {
            unsigned
        } else {
            unsigned << (l0 - 11)
        };

        // strips the first 1 we shifted to bit 52, preserves all other bits.
        let ieee_mantissa = explicit_lead_mantissa & !(1 << 52);
        let composed = sign_bit_placed + exp_bits_placed + ieee_mantissa;

        f64::from_bits(composed)
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn lshift(&self, amount: u8) -> Self {
        //! shifts the inner value left by "amount" bits.
        Self(self.0 << amount)
    }

    pub fn rshift(&self, amount: u8) -> Self {
        //! shifts the inner value right by "amount" bits.
        Self(self.0 >> amount)
    }

    pub fn as_step_fp(self) -> StepFp {
        //! converts self to a StepFp, consuming it.
        if N < STEP_FIXED_POINT_DECIMAL_BITS {
            // if N is less, need to be more precise; inner value grows
            StepFp::with_internal(self.0 << (STEP_FIXED_POINT_DECIMAL_BITS - N))
        } else {
            StepFp::with_internal(self.0 >> (N - STEP_FIXED_POINT_DECIMAL_BITS))
        }
    }

    pub fn as_solar_fp(self) -> SolarFp {
        //! Convert fixed point of any size to SolarFp, preserving the value as closely as possible.
        if N < SOLAR_FIXED_POINT_DECIMAL_BITS {
            // if N is less, need to be more precise; inner value grows
            SolarFp::with_internal(self.0 << (SOLAR_FIXED_POINT_DECIMAL_BITS - N))
        } else {
            SolarFp::with_internal(self.0 >> (N - SOLAR_FIXED_POINT_DECIMAL_BITS))
        }
    }

    pub fn div_by_solar(self, divisor: &SolarFp) -> Self {
        //! divide an FP of any type by a SolarFp. Needed for acceleration calculations.
        //! Implementation assumes SolarFP is the largest type.
        const {
            assert!(
                N >= SOLAR_FIXED_POINT_DECIMAL_BITS,
                "Using div_by_solar on a type larger than SolarFp"
            );
        }
        let bit_shift = N - SOLAR_FIXED_POINT_DECIMAL_BITS;
        let step_divisor = FixedPoint::<N>::with_internal(divisor.0);
        (self / step_divisor).rshift(bit_shift)
    }
}

impl UnitFp {
    pub fn scale_by_other<const N: u8>(self, scalar: FixedPoint<N>) -> FixedPoint<N> {
        //! used in the scaling of unit vectors. Scales by a scalar of varying type, spits that type back out.
        let internal_wide = self.0 as i128 * scalar.0 as i128;

        assert!(internal_wide.abs().leading_zeros() as u8 > 64 - UNIT_FIXED_POINT_DECIMAL_BITS);
        let internal = (internal_wide >> UNIT_FIXED_POINT_DECIMAL_BITS) as i64;
        FixedPoint::<N>(internal)
    }
}

impl<const N: u8> Add for FixedPoint<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.strict_add(rhs.0))
    }
}

impl<const N: u8> Sub for FixedPoint<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.strict_sub(rhs.0))
    }
}

impl<const N: u8> Mul for FixedPoint<N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let wide_ans = (self.0 as i128 * rhs.0 as i128) >> N; // (MR A.2a) i64 -> i128 direct superset.
        assert!(wide_ans.abs() <= i64::MAX as i128); // (MR A.2a) i64 -> i128 direct superset.
        Self(wide_ans as i64) // (MR A.2b) safety assured by the assert! above.
    }
}

impl<const N: u8> Div for FixedPoint<N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // multiply self's intermediate by the unity scale because we need it in the final number and div cancels it.
        let wide_self = (self.0 as i128) << N; // (MR A.2a) i64 -> i128 direct superset.
        let wide_other = (rhs.0) as i128; // (MR A.2a) i64 -> i128 direct superset.
        let wide_ans = wide_self / wide_other;

        assert!(wide_ans.abs() <= i64::MAX as i128); // (MR A.2a) i64 -> i128 direct superset.
        Self((wide_self / wide_other) as i64) // (MR A.2b) safety assured by the assert! above.
    }
}

impl<const N: u8> AddAssign for FixedPoint<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<const N: u8> SubAssign for FixedPoint<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<const N: u8> MulAssign for FixedPoint<N> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl<const N: u8> DivAssign for FixedPoint<N> {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl<const N: u8> Neg for FixedPoint<N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<const N: u8> Display for FixedPoint<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.8}", self.to_f64())
    }
}

impl<const N: u8> std::fmt::Debug for FixedPoint<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FixedPoint<{:02}>, val: {:+.4e}, internal: {:020}",
            N,
            self.to_f64(),
            self.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::FixedPoint;
    type TestFp = FixedPoint<56>; // slightly larger integer range to keep testing easy.

    #[test]
    fn add() {
        assert_eq!(
            TestFp::from_int(2) + TestFp::from_int(3),
            TestFp::from_int(5)
        )
    }

    #[test]
    fn sub() {
        assert_eq!(
            TestFp::from_int(4) - TestFp::from_int(3),
            TestFp::from_int(1)
        )
    }

    #[test]
    fn mul() {
        assert_eq!(
            TestFp::from_int(2) * TestFp::from_int(3),
            TestFp::from_int(6)
        )
    }

    #[test]
    fn div() {
        assert_eq!(
            TestFp::from_int(6) / TestFp::from_int(2),
            TestFp::from_int(3)
        )
    }

    #[test]
    fn from_float() {
        assert_eq!(TestFp::from_f64(1.0).unwrap(), TestFp::from_int(1))
    }

    #[test]
    fn from_float_decimal() {
        assert_eq!(
            TestFp::from_f64(1.5).unwrap(),
            TestFp::from_int(3) / TestFp::from_int(2)
        )
    }

    #[test]
    fn to_float() {
        assert_eq!(TestFp::from_f64(1.5).unwrap().to_f64(), 1.5f64)
    }

    #[test]
    fn abs() {
        assert_eq!(
            TestFp::from_f64_trusted(-1.5).abs(),
            TestFp::from_f64_trusted(1.5)
        )
    }
}
