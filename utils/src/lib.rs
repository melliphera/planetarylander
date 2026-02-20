mod fixed_point;
mod quaternion;
mod sim_sleep;
mod vec3d;

pub mod consts;
pub mod errors;
pub mod message_channels;

pub use errors::*;
pub use fixed_point::{FixedPoint, FloatConversionError, SolarFp, StepFp, UnitFp};
pub use quaternion::Quaternion;
pub use sim_sleep::sim_sleep;
pub use vec3d::{PrintType, SolarVec3D, StepVec3D, UnitVec3D};

// this is for testing!
//mod vec3d_f64;
//pub use vec3d_f64::Vec3Df64 as Vec3D;
