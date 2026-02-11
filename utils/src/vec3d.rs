//! basic 3D vector. Used for movement, forces etc.

use crate::fixed_point::{
    FixedPoint, FloatConversionError, SOLAR_FIXED_POINT_DECIMAL_BITS,
    STEP_FIXED_POINT_DECIMAL_BITS, UNIT_FIXED_POINT_DECIMAL_BITS,
};

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Vec3D<const N: u8>(pub FixedPoint<N>, pub FixedPoint<N>, pub FixedPoint<N>);

pub type UnitVec3D = Vec3D<UNIT_FIXED_POINT_DECIMAL_BITS>;
pub type StepVec3D = Vec3D<STEP_FIXED_POINT_DECIMAL_BITS>;
pub type SolarVec3D = Vec3D<SOLAR_FIXED_POINT_DECIMAL_BITS>;

impl<const N: u8> Default for Vec3D<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: u8> Vec3D<N> {
    pub fn new() -> Self {
        Self(
            FixedPoint::<N>::from_int(0),
            FixedPoint::<N>::from_int(0),
            FixedPoint::<N>::from_int(0),
        )
    }

    pub fn from_floats(x: f64, y: f64, z: f64) -> Result<Self, FloatConversionError> {
        let x = FixedPoint::<N>::from_f64(x)?;
        let y = FixedPoint::<N>::from_f64(y)?;
        let z = FixedPoint::<N>::from_f64(z)?;

        Ok(Self(x, y, z))
    }

    pub const fn from_floats_trusted(x: f64, y: f64, z: f64) -> Self {
        //! const function to be used with magic numbers only to allow compile-time instantiation.
        let x = FixedPoint::<N>::from_f64_trusted(x);
        let y = FixedPoint::<N>::from_f64_trusted(y);
        let z = FixedPoint::<N>::from_f64_trusted(z);

        Self(x, y, z)
    }

    pub fn add(&self, other: &Self) -> Self {
        //! composes two vectors. Used for applying forces between two bodies.
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }

    pub fn sub(&self, other: &Self) -> Self {
        //! subtract other from self.
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }

    pub fn vector_to(&self, other: &Self) -> Self {
        //! get the direction vector from self to other.
        other.sub(self)
    }

    pub fn magnitude(&self) -> FixedPoint<N> {
        //! returns the magnitude of the current vector e.g Vec3D(3, 4, 0).magnitude() == 5.
        //(self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
        let squared_internals: (u128, u128, u128) = (
            (self.0 .0.unsigned_abs() as u128).pow(2),
            (self.1 .0.unsigned_abs() as u128).pow(2),
            (self.2 .0.unsigned_abs() as u128).pow(2),
        ); // (MR A.2b) i64 -> u128, non-negativity enforced by .abs().
        let sum = squared_internals.0 + squared_internals.1 + squared_internals.2;
        assert!(sum.leading_zeros() >= 2); // <=126 bits, sqrt <=63 bits, safe to i64;
        let sqrt = sum.isqrt() as i64; // (MR A.2b) see assert comment above.
        FixedPoint::<N>(sqrt)
    }

    pub fn scale(&self, scale_factor: FixedPoint<N>) -> Self {
        //! scales the vector by a given magnitude.
        Self(
            self.0 * scale_factor,
            self.1 * scale_factor,
            self.2 * scale_factor,
        )
    }

    pub fn scale_down(&self, scale_factor: FixedPoint<N>) -> Self {
        //! divides the vector by a given factor.
        Self(
            self.0 / scale_factor,
            self.1 / scale_factor,
            self.2 / scale_factor,
        )
    }

    pub fn to_unit_vector(self) -> UnitVec3D {
        let shrunk = Vec3D::<UNIT_FIXED_POINT_DECIMAL_BITS>(
            FixedPoint(self.0 .0),
            FixedPoint(self.1 .0),
            FixedPoint(self.2 .0),
        );
        let divisor = shrunk.magnitude();

        shrunk.scale_down(divisor)
    }

    pub fn as_solar(&self) -> SolarVec3D {
        //! converts a vec of any scale to Solar, preserving the represented value as well as possible.
        Vec3D(
            self.0.as_solar_fp(),
            self.1.as_solar_fp(),
            self.2.as_solar_fp(),
        )
    }
}

impl UnitVec3D {
    pub fn scale_from_unit<const N: u8>(self, scalar: FixedPoint<N>) -> Vec3D<N> {
        Vec3D(
            self.0.scale_by_other(scalar),
            self.1.scale_by_other(scalar),
            self.2.scale_by_other(scalar),
        )
    }
}

pub enum PrintType {
    GraphSingle(usize),
    GraphAll,
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn get_test_vecs() -> (Vec3D<56>, Vec3D<56>) {
        //! provides a couple of vectors to use in the test suite.
        (
            Vec3D::from_floats(5.0, 7.0, 10.0).unwrap(),
            Vec3D::from_floats(2.0, 9.0, 15.0).unwrap(),
        )
    }

    #[test]
    pub fn addition() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(v1.add(&v2), Vec3D::from_floats(7.0, 16.00, 25.0).unwrap())
    }

    #[test]
    pub fn subtraction() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(v1.sub(&v2), Vec3D::from_floats(3.0, -2.0, -5.0).unwrap())
    }

    #[test]
    pub fn vector_to() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(
            v1.vector_to(&v2),
            Vec3D::from_floats(-3.0, 2.0, 5.0).unwrap()
        )
    }

    #[test]
    fn scaling() {
        let (v1, _v2) = get_test_vecs();
        assert_eq!(
            v1.scale(FixedPoint::<56>::from_int(10)),
            Vec3D::from_floats(50.0, 70.0, 100.0).unwrap()
        )
    }

    #[test]
    fn unitise() {
        let vec = UnitVec3D::from_floats(1.0, 2.0, 2.0).unwrap();
        let unit = vec.to_unit_vector();
        let unit_spawned = UnitVec3D::from_floats(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0).unwrap();
        println!("{:?}\n{:?}", unit, unit_spawned);
        assert!(
            // greatest difference is less than 2000;
            (unit.0 .0 - unit_spawned.0 .0.abs()).max(
                (unit.1 .0 - unit_spawned.1 .0.abs()).max(unit.2 .0 - unit_spawned.2 .0.abs())
            ) < 2000
        )
    }
}
