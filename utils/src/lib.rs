//! contains primitives, helpers and consts used by agc_rocket and agc_physics.
mod quaternion;
mod thread_clock;
mod vec3d;

pub mod consts;
pub mod errors;
pub mod fixed_point;
pub mod message_channels;

pub use errors::*;
pub use fixed_point::{FixedPoint, FloatConversionError, SolarFp, StepFp, UnitFp};
pub use quaternion::Quaternion;
pub use thread_clock::ThreadClock;
pub use vec3d::{SolarVec3D, StepVec3D, UnitVec3D};

// this is for testing!
//mod vec3d_f64;
//pub use vec3d_f64::Vec3Df64 as Vec3D;
