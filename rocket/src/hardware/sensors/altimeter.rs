//! contains the struct definition for the Altimeter. This sensor works between 40km and gives the distance to the surface.

use std::time::Instant;

use rand::Rng;
use tokio::sync::watch;

use agc_physics::planets::Body;
use agc_utils::{SolarFp, SolarVec3D, UnitFp};
//use agc_utils::Vec3D;

use super::{
    SensorReading,
    _SensorState::{self, *},
};

const _ALTIMETER_DRIFT_BOUNDS: UnitFp = UnitFp::from_f64_trusted(0.000375); // m/s; want ~10-100m/year so must be tiny.

pub struct _AltimeterData {
    state: _SensorState,
    variance: UnitFp,                      // Operational deviation from true values.
    last_reading: SensorReading<SolarFp>, // last reading collected by the device.
    drift: SolarFp,                        // constantly growing deviation from real values
    drift_rate: SolarFp, // rate at which drift increases (per second). Randomised between +/-ALTIMETER_DRIFT_BOUNDS on startup/reboot
    polling_delay: f64,
    max_range: SolarFp,
    send_channel: watch::Sender<SensorReading<SolarFp>>, //
}

impl _AltimeterData {
    pub fn _poll(&mut self, location: SolarVec3D, target: &Body) {
        //! internal polling of data. Error type is just log/debug str as within the scope of the program, sensors need to fail silently.
        //! note that this does not send any data anywhere, it just updates the internally held value.
        match self.state {
            Operational => {
                let true_distance = location.vector_to(&target.position).magnitude();
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance).scale_by_other(true_distance)
                            + self.drift,
                        time: Instant::now(), // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Variant => {
                let true_distance = location.vector_to(&target.position).magnitude();
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance * UnitFp::from_int(5))
                            .scale_by_other(true_distance) + self.drift,
                        time: Instant::now(), // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Garbage => {
                self.last_reading = SensorReading {
                    data: SolarFp::with_internal(rand::thread_rng().gen()), // garbage data
                    time: Instant::now(),
                }
            }
            Frozen(_) | Rebooting(_) => {
                unreachable!("The altimeter will never poll itself while frozen or rebooting.")
            }
        }
    }
}
