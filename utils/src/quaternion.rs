use crate::fixed_point::{FloatConversionError, UnitFp, UNIT_FIXED_POINT_DECIMAL_BITS};
use crate::vec3d::Vec3D;

/// the unit quaternion (1, 0, 0, 0) is defined herein as pointing in the postive x direction, with a roll such that the body's secondary axis is +z.
/// to provide a human example, the human oriented (1, 0, 0, 0) would be lying down with their head on the +x end, looking upwards.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion(UnitFp, UnitFp, UnitFp, UnitFp);

#[derive(Debug, Clone, Copy, PartialEq)]
enum _QuaternionError {
    NotUnit,
    BadFloat(FloatConversionError),
}

impl From<FloatConversionError> for _QuaternionError {
    fn from(value: FloatConversionError) -> Self {
        // if a FloatConversionError propagates from generation, pass it up as QuaternionError::BadFloat
        _QuaternionError::BadFloat(value)
    }
}

const _ERR_EPSILON: UnitFp = UnitFp::from_f64_trusted(1e-3);

impl Quaternion {
    fn _new(w: UnitFp, x: UnitFp, y: UnitFp, z: UnitFp) -> Result<Quaternion, _QuaternionError> {
        //! creates a new Quaternion; checking the values provided produce a unit quaternion.
        let mag = w * w + x * x + y * y + z * z;
        if (mag - UnitFp::from_int(1)).abs() < _ERR_EPSILON {
            Ok(Quaternion(w, x, y, z))
        } else {
            Err(_QuaternionError::NotUnit)
        }
    }

    fn _from_floats(w: f64, x: f64, y: f64, z: f64) -> Result<Quaternion, _QuaternionError> {
        let (wufp, xufp, yufp, zufp) = (
            UnitFp::from_f64(w)?,
            UnitFp::from_f64(x)?,
            UnitFp::from_f64(y)?,
            UnitFp::from_f64(z)?,
        );
        Quaternion::_new(wufp, xufp, yufp, zufp)
    }

    fn _mult(&self, other: &Self) -> Self {
        //! produces self * other. Remember that order matters.
        Self(
            self.0 * other.0 - self.1 * other.1 - self.2 * other.2 - self.3 * other.3,
            self.0 * other.1 + self.1 * other.0 + self.2 * other.3 - self.3 * other.2,
            self.0 * other.2 - self.1 * other.3 + self.2 * other.0 + self.3 * other.1,
            self.0 * other.3 + self.1 * other.2 - self.2 * other.1 + self.3 * other.0,
        )
    }

    fn _conjugated(&self) -> Self {
        //! returns the conjugate pair of self.
        Self(self.0, -self.1, -self.2, -self.3)
    }

    fn _from_vector(vector: Vec3D<UNIT_FIXED_POINT_DECIMAL_BITS>) -> Self {
        //! convert vector to quaternion; used in the conversion of quaternion to vec.
        Quaternion(UnitFp::from_int(0), vector.0, vector.1, vector.2)
    }

    fn _to_forward_vector(&self) -> Vec3D<UNIT_FIXED_POINT_DECIMAL_BITS> {
        //! find the unit-length forward vector of the quaternion.
        //! Done by producing a quaternion representing the principal vector and then multiplying the product by q's conjugate.
        //! Multiplication is done by hand rather than via mult() as many of the operations cancel.
        let qv = Self(-self.1, self.0, self.3, -self.2); // self * (0, 1, 0, 0)
        let qvq_inv = qv._mult(&self._conjugated());
        Vec3D(qvq_inv.1, qvq_inv.2, qvq_inv.3)
    }

    #[cfg(test)]
    fn equal_within_epsilon(&self, other: &Self) -> bool {
        // return true if all fields values are within ERR_EPSILON.
        (self.0 - other.0).abs() < _ERR_EPSILON
            && (self.1 - other.1).abs() < _ERR_EPSILON
            && (self.2 - other.2).abs() < _ERR_EPSILON
            && (self.3 - other.3).abs() < _ERR_EPSILON
    }
}

#[cfg(test)]
mod quaternion_tests {
    use crate::{vec3d::Vec3D, FloatConversionError};

    use super::{Quaternion, UnitFp, _QuaternionError};

    #[test]
    fn test_new_valid() {
        // this creation passes validation because its magnitude is 1
        let ff = Quaternion::_from_floats(0.0, 0.6, 0.8, 0.0);
        let expected = Ok(Quaternion(
            UnitFp::from_f64_trusted(0.0),
            UnitFp::from_f64_trusted(0.6),
            UnitFp::from_f64_trusted(0.8),
            UnitFp::from_f64_trusted(0.0),
        ));
        println!("float: {:?}\nexpec: {:?}", ff, expected);
        assert_eq!(ff, expected)
    }

    #[test]
    fn test_new_invalid() {
        // magnitude too far from 1, fail
        assert_eq!(
            Quaternion::_from_floats(1.0, 1.0, 0.0, 0.0),
            Err(_QuaternionError::NotUnit)
        );

        // bad float fed in, float fails bounds check
        assert_eq!(
            Quaternion::_from_floats(1.0, 3.0, 1299.0, 3.0),
            Err(_QuaternionError::BadFloat(
                FloatConversionError::OutOfBounds
            ))
        )
    }

    #[test]
    fn test_mult() {
        // multiply two quaternions together, validate output is correct against external calculation (within ERR_EPSILON on each field).
        let multed = Quaternion::_from_floats(0.3, 0.6, 0.5, 0.547722558)
            .unwrap()
            ._mult(&Quaternion::_from_floats(0.8, 0.1, 0.5, 0.316227766).unwrap());
        let result = Quaternion(
            UnitFp::from_f64_trusted(-0.2432050809041),
            UnitFp::from_f64_trusted(0.394252604),
            UnitFp::from_f64_trusted(0.4150355962),
            UnitFp::from_f64_trusted(0.7830463762),
        );
        println!("{:?}\n{:?}", multed, result);
        assert!(multed.equal_within_epsilon(&result))
    }

    #[test]
    fn test_conjugate() {
        let quat_test = Quaternion::_from_floats(0.0, 0.6, 0.8, 0.0).unwrap();
        assert_eq!(
            quat_test._conjugated(),
            Quaternion::_from_floats(0.0, -0.6, -0.8, -0.0).unwrap()
        );
    }

    #[test]
    fn test_from_vector() {
        let test_vector = Vec3D::from_floats(0.6, 0.8, 0.0).unwrap();
        assert_eq!(
            Quaternion::_from_vector(test_vector),
            Quaternion::_from_floats(0.0, 0.6, 0.8, 0.0).unwrap()
        )
    }

    #[test]
    fn test_to_forward_vector() {
        assert_eq!(
            Quaternion::_from_floats(1.0, 0.0, 0.0, 0.0)
                .unwrap()
                ._to_forward_vector(),
            Vec3D::from_floats(1.0, 0.0, 0.0).unwrap()
        )
    }
}
