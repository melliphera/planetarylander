//! Contains the Rocket struct. This serves as the interface between real data in the orbit (true position, velocity etc) and the faulty instruments, sensors and flight controller.
//! As such, these instruments will need to query Rocket for information from time to time; e.g. the altimeter needs to know the true distance to the surface in order to produce an unknown one.
//! Instruments must never store values acquired directly from Rocket without processing them to add their own inaccuracy first (this would be cheating!)

use agc_utils::{Quaternion, SolarVec3D, StepVec3D};

pub struct _Rocket {
    position: SolarVec3D,
    velocity: StepVec3D,
    orientation: Quaternion,
}
