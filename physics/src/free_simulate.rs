//! Contains consts, enums and methods for free-running a System for a fixed amount of time with fixed timestep.
//! Intention is to use this for orbit plotting.
use agc_utils::SimulationError;

/// 200 steps per day
pub const TIME_STEP: f64 = 43.20;

/// 2 earth years; duration of full simulations done by System.simulate()
pub const SIM_TIME: f64 = 86400.0 * 365.25 * 2.0; //

/// Number of steps within the simulation time.
pub const STEPS: usize = (SIM_TIME / TIME_STEP) as usize; // (MR A.2b) Both of the above must be positive. Practical use of this code explicitly requires an upper bound of this value well below the usize limit.

/// Instruction for System.simulate().
pub enum PrintType {
    /// Prints position data for BODIES[usize] where usize is the contained value
    GraphSingle(usize),

    /// Prints position data for all bodies.
    GraphAll,
}

impl crate::System {
    /// simulates the System as fast as it possibly can. uses the constants STEPS,
    pub fn simulate(
        &mut self,
        print_type: PrintType,
        print_interval: usize,
    ) -> Result<(), SimulationError> {
        let mut energy: f64;
        let mut prev_energy = 0f64;
        let mut max_energy = -f64::MAX; // value selected to ensure first actual value overwrites.
        let mut min_energy = f64::MAX; // value selected to ensure first actual value overwrites.

        for step in 0..STEPS {
            if step % print_interval == 0 {
                // print data for current step.
                match print_type {
                    PrintType::GraphSingle(p_index) => {
                        let pb = match self.bodies.get(p_index) {
                            Some(ind) => ind,
                            None => {
                                println!("Trying to print data on an invalid body!");
                                return Err(SimulationError::BadPrintIndex);
                            }
                        };
                        println!(
                            "{}, {}, {}, {}",
                            pb.name[..3].to_uppercase(),
                            pb.position.0,
                            pb.position.1,
                            pb.position.2
                        )
                    }
                    PrintType::GraphAll => {
                        for pb in self.bodies.iter() {
                            println!(
                                "{}, {}, {}, {}",
                                pb.name[..3].to_uppercase(),
                                pb.position.0,
                                pb.position.1,
                                pb.position.2
                            )
                        }
                    }
                }
            }

            energy = self.step_time_forwards(TIME_STEP, None)?; // does the logical part, moving and accelerating bodies.

            if step > 0 && step % print_interval == 0 {
                println!(
                    "System energy: {:.6e}\tchange: {:.6e} ({:+.2}%)",
                    energy,
                    energy - prev_energy,
                    (energy / prev_energy - 1.0) * 100.0
                )
            }
            // energy logging/maintenance
            prev_energy = energy;
            max_energy = max_energy.max(energy);
            min_energy = min_energy.min(energy)
        }
        println!(
            "\nmin energy: {:.4e}\nmax energy: {:.4e}\ndeviation: {}%",
            min_energy,
            max_energy,
            (max_energy / min_energy - 1.0) * 100.0
        );
        Ok(())
    }
}
