use std::time::{Duration, Instant};

use crate::{
    consts::{TICK_DELAY, TICK_DELAY_AS_DURATION},
    BroadcastError,
};

/// Manages both real-time and sim-time synchronisation of threads.
/// Each thread has its own copy of the same ThreadClock, and instructions to sync/change behaviour are handled by the clock.
/// Provides an "end_tick()" method that sleeps until the start of the next tick.
/// Provides an "instrument_sleep() method that skips all core logic for a tick without blocking the tick."
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub struct ThreadClock {
    pub epoch: Instant, // used to keep ticks synced
    current_tick_start: Instant,
    tick_length: Duration,
    pub counter: u32,           // number of ticks passed
    pub sim_time_per_tick: f64, // sim time passed (in seconds) per simulation tick.
}

impl ThreadClock {
    pub fn new(initial_time_scale: f64) -> Self {
        //! creates a new instance of ThreadClock.
        //! initial_time_scale is a variable because the "natural" default value of 1 breaks the simulation.
        let init_instant = Instant::now();
        let stpt = TICK_DELAY * initial_time_scale;
        assert!(
            stpt > 0.5,
            "Sim time per tick must be greater than 0.5s. Current: {stpt:.4}s"
        );
        Self {
            epoch: init_instant,
            current_tick_start: init_instant,
            tick_length: TICK_DELAY_AS_DURATION,
            counter: 0,
            sim_time_per_tick: stpt,
        }
    }

    pub fn end_tick(&mut self) -> Result<(), BroadcastError> {
        //! handles the ending of a tick by incrementing the tick counter and then sleeping to the start of the next tick.
        self.counter = self.counter.saturating_add(1);

        let current_tick_elapsed = self.current_tick_start.elapsed();
        self.current_tick_start = match self
            .epoch
            .checked_add(self.tick_length.saturating_mul(self.counter))
        {
            Some(val) => val,
            None => return Err(BroadcastError::UnlinkedChannel), // bogus error but its okay; tick time failure will always occur on main thread first.
        };

        let until_next_tick = self.tick_length.saturating_sub(current_tick_elapsed);
        std::thread::sleep(until_next_tick);
        Ok(())
    }
}
