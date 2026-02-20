//! Contains instantiation logic for the flight controller. This also manages the creation and thread spawning for other sensors, and their linking to the flight controller.
//! The FlightController is the logical heart of the rocket. The scope of the FlightController is inspired by KSP (emulating the player); not only does it collect information from the sensors,
//! and have authority over the flight hardware, but it's also responsible for controlling the timescale of the simulation; the flightcontroller will understand when it needs precision, and adjust time accordingly.

use crate::hardware::sensors::{Sensor, SensorHandle};
use agc_utils::message_channels::{
    FcMessageOut::{self, *},
    FcMessageReceiver, FcMessageSender,
};
use agc_utils::SolarFp;
use agc_utils::{errors::*, ThreadClock};

use super::sensors;

pub struct FlightController {
    _timescale: SolarFp,
    sys_channel: Option<FcMessageSender>,
    sensor_broadcast_handles: [Option<SensorHandle>; sensors::NUM_SENSORS],
}

impl FlightController {
    pub fn new() -> Self {
        FlightController {
            _timescale: SolarFp::from_int(1),
            sys_channel: None,
            sensor_broadcast_handles: std::array::from_fn(|_| None),
        }
    }

    pub fn create_system_channel(&mut self) -> FcMessageReceiver {
        //! creates the channel to communicate with the System thread,
        let (sx, rx) = std::sync::mpsc::channel();
        self.sys_channel = Some(sx);
        rx
    }

    pub fn create_sensors(&mut self) {
        // Generates all the sensors and handles message threads associated with them.
        let alti_handle = Sensor::Altimeter.generate();
        self.sensor_broadcast_handles[0] = Some(alti_handle);
    }

    pub fn start(&mut self, initial_scale: f64) -> Result<(), SimulationError> {
        //! Sends a signal out to all listeners (who are blocked waiting for it) containing a syncing ThreadClock.
        //! initial_scale represents the time-scale (fast-forwarding factor) that the sim initially runs at.
        let tc = ThreadClock::new(initial_scale);
        self.broadcast(GoSynced(tc))?;
        self.flightloop()?; // point of handover to src/logic/ -  This function can be found in src/logic/mod.rs
        Ok(())
    }

    pub fn broadcast(&self, signal: FcMessageOut) -> Result<(), BroadcastError> {
        //! sends the defined message out on all its channels. used for starting and terminating the simulation.
        match &self.sys_channel {
            Some(ch) => ch.send(signal)?,
            None => return Err(BroadcastError::UnlinkedChannel),
        };
        for opt_sbc in self.sensor_broadcast_handles.iter() {
            match opt_sbc {
                Some(handle) => match handle {
                    SensorHandle::Altimeter(send_channel, _stored) => send_channel.send(signal)?,
                    SensorHandle::InertialPlatform(send_channel, _stored) => {
                        send_channel.send(signal)?
                    }
                },
                None => return Err(BroadcastError::UnlinkedChannel),
            };
        }
        Ok(())
    }
}
