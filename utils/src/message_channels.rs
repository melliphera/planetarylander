//! contains various shorthands for different types of message channel, to have consistent and concise type names across the project.
#![allow(missing_docs)] // crate is well commented and rearranging to fit doc comment requirements would hurt readability.
use std::sync::mpsc;

use crate::{SolarFp, UnitFp};

// used for FC outgoing communications.
pub type FcMessageSender = mpsc::Sender<FcMessageOut>;
pub type FcMessageReceiver = mpsc::Receiver<FcMessageOut>;

// Used for Sensor -> FC and System -> Sensor comms
pub type DataSender<T> = tokio::sync::watch::Sender<SensorReading<T>>;
pub type DataReceiver<T> = tokio::sync::watch::Receiver<SensorReading<T>>;

pub use tokio::sync::watch::channel as watch_channel; // re-export for conciseness and to minimise [dependencies] in other manifests.

/// Represents a single reading from a sensor. Contains the reading data (type: T) and the time it was harvested.
/// Is also used (with a large struct as T) for communications from System to Sensor
#[derive(Clone, Copy, Debug)]
pub struct SensorReading<T> {
    pub data: T,
    pub time: f64, // in-simulation time of data collection.
}

impl<T: Default> SensorReading<T> {
    pub fn new() -> Self {
        SensorReading {
            data: T::default(),
            time: 0.0,
        }
    }
}

impl<T: Default> Default for SensorReading<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// message that can be sent out by the FlightController, either to the System simulation or to sensors.
#[derive(Clone, Copy, Debug)]
pub enum FcMessageOut {
    Heartbeat, // Tells receiver thread "continue as you are". Thread-syncing no-op.
    GoSynced(crate::ThreadClock), // Tells sensor/sys sim to start running mainloop. Carries a ThreadClock instance to keep threads synced.
    Restart, // Tells a faulty sensor to restart, recalibrating its drift. No use for System.
    NewTimescale(SolarFp, SolarFp), // Tells the sensor/system to operate on a new timescale (first value) and sends the current in-sim time for syncing (second value).
    RocketCommand(RC), // TODO - tells the System simulation to change the Rocket's instructions.
    Kill,              // Tells the sensor/system thread to end (end of simulation)
}

#[derive(Clone, Copy, Debug)]
pub enum RC {
    Thrust(UnitFp),
}
