//! contains the struct definitin for the InertialPlatform; a combined accelerometer/gyroscope.

use crate::hardware::sensors::SensorState;
pub struct _InertialPlatformData {
    state: SensorState,
}
