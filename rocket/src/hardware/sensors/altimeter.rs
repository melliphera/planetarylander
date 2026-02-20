//! contains the struct definition for the Altimeter. This sensor works between 40km and gives the distance to the surface.

use std::{thread, time::Instant};

use rand::Rng;

use agc_physics::planets::Body;
use agc_utils::consts::*;
use agc_utils::errors::*;
use agc_utils::{SolarFp, SolarVec3D, StepFp, UnitFp};
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
}

impl Altimeter {
    pub fn start_thread(send_channel: DataSender<SolarFp>, receive_channel: FcMessageReceiver) {
        // instantiate the altimeter itself; assume it starts in perfect condition
        let alti = Altimeter {
            state: SensorState::Operational,
            variance: UnitFp::from_int(0),
            last_reading: SensorReading {
                data: SolarFp::from_int(0),
                time: Instant::now(),
            },
            drift: SolarFp::from_int(0),
            drift_rate: UnitFp::from_int(0),
            polling_delay: StepFp::from_f64_trusted(0.5),
            max_range: SolarFp::from_int(80_000),
            reading_body: 3, // Earth; that's where we're starting.
            send_channel,
            receive_channel,
        };

        let _unused_handle = thread::spawn(|| alti.mainloop());
    }

    fn mainloop(self) -> Result<(), BroadcastError> {
        //! Main operating loop of the altimeter.
        let go_call = self.receive_channel.recv();
        let epoch = match go_call {
            Ok(FcMessageOut::GoSynced(t)) => t,
            Ok(_) | Err(_) => return Err(BroadcastError::WrongInit), // first value received wasn't the right one.
        };
        let mut tick_counter: u32 = 0;
        let mut tick_time = epoch;
        loop {
            // await (blocking) signal from System that simulation is done.

            // watch (non-blocking) signal from FC - it doesn't matter if this is a tick late as it will never be done at critical times.
            if let Ok(transmission) = self.receive_channel.try_recv() {
                match transmission {
                    FcMessageOut::GoSynced(_) => return Err(BroadcastError::BadRuntimeSignal), // bogus signal to be receiving at this point
                    FcMessageOut::Restart => {
                        unimplemented!("Need to write restart function")
                    }
                    FcMessageOut::NewTimescale(_, _) => {
                        unimplemented!("Timescale handling not implemented yet.")
                    }
                    FcMessageOut::Heartbeat => {}
                    FcMessageOut::Kill => break,
                }
            }

            // do actual tick logic here
            if tick_counter.is_multiple_of(128) {
                println!(
                    "Altimeter:\tTicks processed: {tick_counter}\tElapsed: {}",
                    epoch.elapsed().as_secs_f32()
                );
            }

            tick_counter = tick_counter.saturating_add(1);

            let current_tick_elapsed = tick_time.elapsed();
            tick_time = match epoch.checked_add(TICK_DELAY_AS_DURATION.saturating_mul(tick_counter))
            {
                Some(val) => val,
                None => return Err(BroadcastError::UnlinkedChannel), // bogus error but its okay; tick time failure will always occur on main thread first.
            };

            let until_next_tick = TICK_DELAY_AS_DURATION.saturating_sub(current_tick_elapsed);
            std::thread::sleep(until_next_tick);
        }
        println!("Altimeter loop exiting after Kill command.");
        Ok(())
    }

    pub fn poll(&mut self, location: SolarVec3D, target: &Body) {
        //! internal polling of data. Error type is just log/debug str as within the scope of the program, sensors need to fail silently.
        //! note that this does not send any data anywhere, it just updates the internally held value.
        match self.state {
            Operational => {
                let true_distance = location.vector_to(&target.position).magnitude();
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance).scale_by_other(true_distance)
                            + self.drift,
                        time: Instant::now(), // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Variant => {
                let true_distance = location.vector_to(&target.position).magnitude();
                if true_distance < self.max_range {
                    self.last_reading = SensorReading {

                        #[allow(clippy::arithmetic_side_effects)] // it's complaining about the addition. We know for a fact that variance is in bounds.
                        data: (UnitFp::from_int(1) + self.variance * UnitFp::from_int(5))
                            .scale_by_other(true_distance) + self.drift,
                        time: Instant::now(), // TODO create wrapper type for Simulation time.
                    }
                }
            }
            Garbage => {
                self.last_reading = SensorReading {
                    data: SolarFp::with_internal(rand::thread_rng().gen()), // garbage data
                    time: Instant::now(),
                }
            }
            Frozen(_) | Rebooting(_) => {
                unreachable!("The altimeter will never poll itself while frozen or rebooting.")
            }
        }
    }
}
