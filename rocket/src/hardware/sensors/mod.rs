pub mod altimeter;
pub mod inertial_platform; // gyroscope + accelerometer

enum SensorState {
    // All sensors are capable of falling into any of these states.
    Operational, // subject to minimal variance, working as expected. Operational variance is defined during instantiation of the hardware.
    Variant,     // subject to 10x variance compared normal, otherwise all working
    Garbage,     // throws out technically parseable data with truly random values.
    Frozen(f64), // simulates hanging sensor. Carried number is unfreeze time.
    Rebooting(f64), // triggered by FlightController. Reverts to Operational after time.
}
