//! contains the struct definitin for the InertialPlatform; a combined accelerometer/gyroscope.

use crate::hardware::sensors::_SensorState;
pub struct _InertialPlatformData {
    state: _SensorState,
}
