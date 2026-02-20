use std::time::Duration;

// for orbital plotting
pub const TIME_STEP: f64 = 43.20; // 200 steps per day
pub const SIM_TIME: f64 = 86400.0 * 365.25 * 2.0; // 2 earth years; duration of full simulations done by System.simulate()
pub const STEPS: usize = (SIM_TIME / TIME_STEP) as usize; // (MR A.2b) Both of the above must be positive. Practical use of this code explicitly requires an upper bound of this value well below the usize limit.

pub const TICK_RATE: u16 = 8;
pub const TICK_DELAY: f64 = 1.0f64 / TICK_RATE as f64;
pub const TICK_DELAY_AS_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE as u64);
