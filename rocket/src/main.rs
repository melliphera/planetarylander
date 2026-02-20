//! Loads a solar system from the agc-physics crate, and simulates the movements of a heavily constrained rocket around that system. See rocket-constraints.txt in the crate root for full details.
mod hardware;
mod logic;

use crate::hardware::flight_controller::FlightController;
use agc_utils::errors::SimulationError;

//use crate::hardware::rocket::Rocket;

fn main() -> Result<(), SimulationError> {
    let mut fc = FlightController::new();
    let system_receiver = fc.create_system_channel();

    // spawn sensors and link to FC
    fc.create_sensors();

    // create channel bundle for system <-> sensor
    agc_physics::System::spawn_live_thread(system_receiver)?;

    // broadcast to all threads to start the simulation. Carries initial timescale data.
    let bc_result = fc.start(128.0);
    match bc_result {
        Ok(()) => {}
        Err(_) => return Err(SimulationError::ThreadConnectionError), // this doesnt feel like an appropriate error to pass back.
    }

    Ok(())
}
