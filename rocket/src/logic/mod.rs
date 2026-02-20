use std::time::Duration;

use agc_utils::errors::SimulationError;

use crate::FlightController;

impl FlightController {
    pub(super) fn flightloop(&mut self) -> Result<(), SimulationError> {
        println!(
            "Flight logic activating (Currently unimplemented). Sending kill signal in 20 seconds."
        );
        std::thread::sleep(Duration::from_secs(20));
        self.broadcast(agc_utils::message_channels::FcMessageOut::Kill)?;
        Ok(())
    }
}
