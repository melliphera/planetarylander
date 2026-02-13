use std::time::Instant;

use agc_utils::{Quaternion, SolarFp, StepVec3D};
use tokio::sync::watch::Receiver;

pub mod altimeter;
pub mod inertial_platform; // gyroscope + accelerometer

/// enum wrapper to enable iteration and single-array storage of different sensors.
/// the enum itself contains a sender and a receiver.
/// creating an instance of the enum involves spawning a thread which the sender independently runs on.
/// the flight controller sends time information to the thread via a blocking sender.
/// the sender then simulates time 
/// 
pub enum Sensor {
    Altimeter(Sender<SolarFp>, Receiver<SensorReading<SolarFp>>),
    InertialPlatform(Sender<SolarFp>, Receiver<SensorReading<(Quaternion, StepVec3D)>>),
}

impl Sensor {
    pub fn generate() -> Self {
        // spawns a new thread which 
    }
}

pub enum _SensorState {
    /// All sensors are capable of falling into any of these states.
    Operational, // subject to minimal variance, working as expected. Operational variance is defined during instantiation of the hardware.
    Variant,        // subject to 10x variance compared normal, otherwise all working
    Garbage,        // throws out technically parseable data with truly random values.
    Frozen(f64),    // simulates hanging sensor. Carried number is unfreeze time.
    Rebooting(f64), // triggered by FlightController. Reverts to Operational after time.
}

pub struct SensorReading<T> {
    /// Represents a single reading from a sensor. Contains the reading data (type: T) and the time it was harvested.
    data: T,
    time: Instant,
}
