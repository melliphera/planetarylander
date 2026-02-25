//!Contains consts for agc_rocket and agc_utils.
use std::time::Duration;

/// tick rate (ticks per second) at which simulations are run.
pub const TICK_RATE: u16 = 128;

/// time between each tick in seconds.
pub const TICK_DELAY: f64 = 1.0f64 / TICK_RATE as f64;

/// Duration object representing the time between each tick.
pub const TICK_DELAY_AS_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE as u64);
