//! Loads a solar system from the agc-physics crate, and simulates the movements of a heavily constrained rocket around that system. See rocket-constraints.txt in the crate root for full details.
use agc_physics::orbit::SimulationError;

mod hardware;
mod logic;

fn main() -> Result<(), SimulationError> {
    let mut system = agc_physics::System::create();
    let before = std::time::Instant::now();
    system.simulate(agc_utils::PrintType::GraphSingle(3), 2000)?;
    println!("Time taken: {:.2}s", before.elapsed().as_secs_f32());
    Ok(())
}
