//! Loads a solar system from the agc-physics crate, and simulates the movements of a heavily constrained rocket around that system. See rocket-constraints.txt in the crate root for full details.
use agc_physics::orbit::SimulationError;

mod hardware;
mod logic;

fn main() -> Result<(), SimulationError> {
    agc_physics::simulate(agc_utils::PrintType::GraphSingle(3), 2000)
}
