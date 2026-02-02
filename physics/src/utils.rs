//! basic 3D vector. Used for movement, forces etc.    
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Vec3D(pub f64, pub f64, pub f64);

impl Vec3D {
    pub fn new() -> Self { Self(0.0, 0.0, 0.0)}

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

    pub fn magnitude(&self) -> f64 {
        //! returns the magnitude of the current vector e.g Vec3D(3, 4, 0).magnitude() == 5.
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }

    pub fn scale(&self, scale_factor: f64) -> Self {
        //! scales the vector by a given magnitude.
        Self(
            self.0 * scale_factor,
            self.1 * scale_factor,
            self.2 * scale_factor,
        )
    }
}

pub enum PrintType {
    GraphSingle(usize),
    GraphAll
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn get_test_vecs() -> (Vec3D, Vec3D) {
        //! provides a couple of vectors to use in the test suite.
        (Vec3D(5.0, 7.0, 10.0), Vec3D(2.0, 9.0, 15.0))
    }

    #[test]
    pub fn addition() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(v1.add(&v2), Vec3D(7.0, 16.00, 25.0))
    }

    #[test]
    pub fn subtraction() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(v1.sub(&v2), Vec3D(3.0, -2.0, -5.0))
    }

    #[test]
    pub fn vector_to() {
        let (v1, v2) = get_test_vecs();
        assert_eq!(v1.vector_to(&v2), Vec3D(-3.0, 2.0, 5.0))
    }
}
