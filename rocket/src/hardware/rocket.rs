//! Contains the Rocket struct. This serves as the interface between real data in the orbit (true position, velocity etc) and the faulty instruments, sensors and flight controller.
//! As such, these instruments will need to query Rocket for information from time to time; e.g. the altimeter needs to know the true distance to the surface in order to produce an unknown one.
//! Instruments must never store values acquired directly from Rocket without processing them to add their own inaccuracy first (this would be cheating!)

use agc_utils::{Quaternion, SolarVec3D, StepVec3D};

pub struct Rocket {
    position: SolarVec3D,
    velocity: StepVec3D,
    orientation: Quaternion,
}

impl Rocket {
    pub fn new() -> Self {
        // spawns a new rocket; positioned on the Earth's equator on the far side of the sun, at epoch.
        Self {
            position: SolarVec3D::from_floats_trusted(
                -2.521202042891241E+10,
                1.449341963234861E+11,
                -6.164432685255142E+05,
            ),
            velocity: StepVec3D::from_floats_trusted(
                -3.025977219503563E+04,
                -5.280684209504152E+03,
                3.173286860330564E+01,
            ),
            orientation: match Quaternion::from_floats(
                0.643668685702801,
                0.0,
                3.255040401211003E-06,
                0.765304268271799,
            ) {
                Ok(q) => q,
                Err(_) => unreachable!("Quaternion has been precomputed."),
            },
        }
    }
}
