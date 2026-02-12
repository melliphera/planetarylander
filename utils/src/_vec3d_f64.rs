//! basic 3D vector. Used for movement, forces etc.    
// TEST VERSION THAT USES F64. THIS IS BANNED IN THE FINAL PROJECT.
use crate::{FixedPoint, fixed_point::FloatConversionError};

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Vec3Df64(pub f64, pub f64, pub f64);

type Vec3D = Vec3Df64; // this is cooked but its for testing.

// lord forgive me for what I'm about to do (justification is to get this number as const)
const TWO_TO_TWENTY: f64 = 1048576.0;
const TWO_TO_SIXTEEN: f64 = 65536.0;
pub const SOLAR_FP_MAX_VAL: f64 = TWO_TO_TWENTY * TWO_TO_TWENTY * TWO_TO_SIXTEEN; // 2^56.

impl Default for Vec3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Vec3D {
    pub fn new() -> Self {
        Self(
            0.0, 
            0.0, 
            0.0
        )
    }

    pub fn from_floats(x: f64, y: f64, z: f64) -> Result<Self, FloatConversionError> {
        // keeping the interface the same as our real vecs
        Ok(Self(x, y, z))
    }

    fn max_guard(&self) {
        // just prints a warning if any float values are higher than our limit
        for val in [self.0, self.1, self.2].iter() {
            if val.abs() > SOLAR_FP_MAX_VAL {
                panic!("WARNING: Vec absolute value exceeded fixed point maximum! {:?}", self)
            }
        }
    }


    pub fn add(&self, other: &Self) -> Self {
        //! composes two vectors. Used for applying forces between two bodies.
        let out = Self(self.0 + other.0, self.1 + other.1, self.2 + other.2);
        out.max_guard();
        return out
    }

    pub fn sub(&self, other: &Self) -> Self {
        //! subtract other from self.
        let out = Self(self.0 - other.0, self.1 - other.1, self.2 - other.2);
        out.max_guard();
        return  out;
    }

    pub fn vector_to(&self, other: &Self) -> Self {
        //! get the direction vector from self to other.
        let out = other.sub(self);
        out.max_guard();
        return out;
    }

    pub fn magnitude(&self) -> f64 {
        //! returns the magnitude of the current vector e.g Vec3D(3, 4, 0).magnitude() == 5.
        let big = self.0 * self.0 + self.1 * self.1 + self.2 * self.2;
        if big > SOLAR_FP_MAX_VAL {
            println!("WARNING: Vec magnitude absolute value exceeded fixed point maximum! {:?}, mag={}, limit_ratio={}", self, big, big/SOLAR_FP_MAX_VAL);
        };
        big.sqrt()
    }

    pub fn scale(&self, scale_factor: f64) -> Self {
        //! scales the vector by a given magnitude.
        let out = Self(
            self.0 * scale_factor,
            self.1 * scale_factor,
            self.2 * scale_factor,
        );
        out.max_guard();
        out
    }
}

pub enum PrintType {
    GraphSingle(usize),
    GraphAll,
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn get_test_vecs() -> (Vec3Df64, Vec3Df64) {
        //! provides a couple of vectors to use in the test suite.
        (Vec3D::from_floats(5.0, 7.0, 10.0).unwrap(), Vec3D::from_floats(2.0, 9.0, 15.0).unwrap())
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
        assert_eq!(v1.vector_to(&v2), Vec3D::from_floats(-3.0, 2.0, 5.0).unwrap())
    }
}
