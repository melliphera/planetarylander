use agc_physics::orbit::SystemData;
use agc_utils::{Quaternion, SolarFp, StepVec3D};
use std::sync::mpsc;

pub mod altimeter;
pub mod inertial_platform; // gyroscope + accelerometer

use agc_utils::message_channels::*;

use crate::hardware::sensors::altimeter::Altimeter;

pub const NUM_SENSORS: usize = 1; // number of currently implemented sensors. Used in a few array bounds and safety checks (to ensure all sensors are linked).

pub enum Sensor {
    // this is just used to have named sensors without using strings. generate() is impl'd on this enum for natural semantics.
    Altimeter,
    InertialPlatform,
}

/// enum wrapper to enable iteration and single-array storage of different sensors.
/// the enum itself contains a sender and a receiver.
/// creating an instance of the enum involves spawning a thread which the sender independently runs on.
/// the flight controller can send information via the FcMessageOut sender, which can contain one of two things;
/// 1) an amount of time that the flightcontroller wishes to advance. This results in the sensor simulating that amount of time passing and sending back results.
/// 2) timescale information; such that the sensor thread's timescale stays synced with the rest of the simulation.
///
/// the sender then simulates time
pub enum SensorHandle {
    Altimeter(FcMessageSender, DataReceiver<SolarFp>),
    InertialPlatform(FcMessageSender, DataReceiver<(Quaternion, StepVec3D)>),
}

impl Sensor {
    pub fn generate(&self, sys_receiver: DataReceiver<SystemData>) -> SensorHandle {
        //! spawns a new thread which operates the sensor runtime.

        // create broadcast pair; not specific to each sensor.
        let (broadcast_sender, broadcast_receiver) = mpsc::channel::<FcMessageOut>();

        match self {
            Sensor::Altimeter => {
                // create sensor-specific instrumnent result pair.
                let (data_sender, data_receiver) =
                    watch_channel::<SensorReading<SolarFp>>(SensorReading {
                        data: SolarFp::from_int(0),
                        time: 0.0,
                    });

                // start the thread itself
                Altimeter::start_thread(data_sender, broadcast_receiver, sys_receiver);

                // return the handle
                SensorHandle::Altimeter(broadcast_sender, data_receiver)
            }
            Sensor::InertialPlatform => {
                unimplemented!()
            }
        }
    }
}

pub enum SensorState {
    /// All sensors are capable of falling into any of these states.
    Operational, // subject to minimal variance, working as expected. Operational variance is defined during instantiation of the hardware.
    Variant,        // subject to 10x variance compared normal, otherwise all working
    Garbage,        // throws out technically parseable data with truly random values.
    Frozen(f64),    // simulates hanging sensor. Carried number is unfreeze time.
    Rebooting(f64), // triggered by FlightController. Reverts to Operational after time.
}
