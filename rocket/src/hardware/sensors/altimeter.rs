//! contains the struct definition for the Altimeter. This sensor works between 40km and gives the distance to the surface.

use std::sync::mpsc::TryRecvError;
use std::thread;

use agc_physics::orbit::SystemData;
use rand::Rng;

use agc_utils::{errors::*, ThreadClock};
use agc_utils::{SolarFp, StepFp, UnitFp};
//use agc_utils::Vec3D;

use super::{
    SensorReading,
    SensorState::{self, *},
};

use agc_utils::message_channels::*;

const ALTIMETER_DRIFT_BOUNDS: UnitFp = UnitFp::from_f64_trusted(0.000375); // m/s; want ~10-100m/year so must be tiny.

pub struct Altimeter {
    state: SensorState,
    variance: UnitFp,                     // Operational deviation from true values.
    last_reading: SensorReading<SolarFp>, // last reading collected by the device.
    drift: SolarFp,                       // constantly growing deviation from real values
    drift_rate: UnitFp, // rate at which drift increases (per second). Randomised between +/-ALTIMETER_DRIFT_BOUNDS on startup/reboot
    polling_delay: StepFp, // delay (in secs) between each poll.
    max_range: SolarFp, // maximum range at which target polls aren't garbage.
    reading_body: usize, // index of the body to read distance from in the bodies list.

    send_channel: DataSender<SolarFp>, // send channel for SensorReading<SolarFp>.
    receive_channel: FcMessageReceiver, // receive channel for ToSensorBroadcast
    data_receive_channel: DataReceiver<SystemData>, // receive channel for raw data from System.
}

impl Altimeter {
    pub fn start_thread(
        send_channel: DataSender<SolarFp>,
        receive_channel: FcMessageReceiver,
        data_receive_channel: DataReceiver<SystemData>,
    ) {
        // instantiate the altimeter itself; assume it starts in perfect condition
        let alti = Altimeter {
            state: SensorState::Operational,
            variance: UnitFp::from_int(0),
            last_reading: SensorReading {
                data: SolarFp::from_int(0),
                time: 0.0,
            },
            drift: SolarFp::from_int(0),
            drift_rate: UnitFp::from_int(0),
            polling_delay: StepFp::from_f64_trusted(0.5),
            max_range: SolarFp::from_int(80_000),
            reading_body: 3, // Earth; that's where we're starting.
            send_channel,
            receive_channel,
            data_receive_channel,
        };

        let _unused_handle = thread::spawn(move || alti.mainloop());
    }

    fn mainloop(self) -> Result<(), BroadcastError> {
        //! Main operating loop of the altimeter.
        let go_call = self.receive_channel.recv();
        let mut thread_clock = match go_call {
            Ok(FcMessageOut::GoSynced(t)) => t,
            Ok(_) | Err(_) => return Err(BroadcastError::WrongInit), // first value received wasn't the right one.
        };
        loop {
            // watch (non-blocking) signal from FC - it doesn't matter if this is a tick late as it will never be done at critical times.
            match self.receive_channel.try_recv() {
                // Sensor-appropriate messages
                Ok(FcMessageOut::Restart) => {
                    unimplemented!("Need to write restart function")
                }
                Ok(FcMessageOut::NewTimescale(_, _)) => {
                    unimplemented!("Timescale handling not implemented yet.")
                }

                // Control messages
                Ok(FcMessageOut::Heartbeat) => {}
                Ok(FcMessageOut::Kill) => break, // Graceful end-of-simulation termination.

                // Bad messages
                Ok(FcMessageOut::GoSynced(_)) => return Err(BroadcastError::BadRuntimeSignal), // bogus signal to be receiving at this point
                Ok(FcMessageOut::RocketCommand(_)) => return Err(BroadcastError::BadRuntimeSignal), // Sensors shouldn't receive this signal.

                // error types
                Err(TryRecvError::Empty) => {} // No message received, continue as normal.
                Err(_) => return Err(BroadcastError::DeallocatedChannel), // Actual error (sender has been deallocated), propagate.
            }

            // do actual tick logic here
            let true_earth_dist = *self.data_receive_channel.borrow();
            println!("EAR-dist: {}m", true_earth_dist.data.altitude);

            thread_clock.end_tick()?;
        }
        println!("Altimeter loop exiting after Kill command.");
        Ok(())
    }

    pub fn poll(&mut self, true_distance: SolarFp, clock: ThreadClock) {
        //! internal polling of data. Error type is just log/debug str as within the scope of the program, sensors need to fail silently.
        //! note that this does not send any data anywhere, it just updates the internally held value.
        match self.state {
            Operational => {
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance).scale_by_other(true_distance)
                            + self.drift,
                        time: clock.sim_time, // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Variant => {
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance * UnitFp::from_int(5))
                            .scale_by_other(true_distance) + self.drift,
                        time: clock.sim_time, // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Garbage => {
                self.last_reading = SensorReading {
                    data: SolarFp::with_internal(rand::thread_rng().gen()), // garbage data
                    time: clock.sim_time, // TODO create wrapper type for Simulation time.
                }
            }
            Frozen(_) | Rebooting(_) => {
                unreachable!("The altimeter will never poll itself while frozen or rebooting.")
            }
        }
    }
}
