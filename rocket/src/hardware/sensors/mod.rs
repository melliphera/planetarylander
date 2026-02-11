use std::time::Instant;

pub mod altimeter;
pub mod inertial_platform; // gyroscope + accelerometer

enum _Sensor {
    /// enum wrapper to enable iteration and single-array storage of different sensors.
    Altimeter(altimeter::_AltimeterData),
    InertialPlatform(inertial_platform::_InertialPlatformData),
}

enum _SensorState {
    /// All sensors are capable of falling into any of these states.
    Operational, // subject to minimal variance, working as expected. Operational variance is defined during instantiation of the hardware.
    Variant,        // subject to 10x variance compared normal, otherwise all working
    Garbage,        // throws out technically parseable data with truly random values.
    Frozen(f64),    // simulates hanging sensor. Carried number is unfreeze time.
    Rebooting(f64), // triggered by FlightController. Reverts to Operational after time.
}

struct _SensorReading<T> {
    /// Represents a single reading from a sensor. Contains the reading data (<T>) and the time it was harvested.
    data: T,
    time: Instant,
}
