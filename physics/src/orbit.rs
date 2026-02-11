//! This file is responsible for the time-step simulation to produce orbital motion.
use std::f64;

use crate::planets::{Body, BODIES, N_BODIES};
use agc_utils::{FloatConversionError, PrintType, SolarFp, StepFp, StepVec3D};

const TIME_STEP: f64 = 432.0; // 200 steps per day
const SIM_TIME: f64 = 86400.0 * 365.25 * 2.0; // 2 earth years
const STEPS: usize = (SIM_TIME / TIME_STEP) as usize; // (MR A.2b) Both of the above must be positive. Practical use of this code explicitly requires an upper bound of this value well below the usize limit.

#[derive(Debug)]
pub enum SimulationError {
    BadTimeStep,
    BadPrintIndex,
}

impl From<FloatConversionError> for SimulationError {
    fn from(_value: FloatConversionError) -> Self {
        Self::BadTimeStep
    }
}

pub fn simulate(print_type: PrintType, print_interval: usize) -> Result<(), SimulationError> {
    let mut bodies = BODIES; // copy for mutability
    let bodies_immutable = BODIES; // copy for clippy linting.

    for body in bodies.iter_mut() {
        body.fill_influencers(&bodies_immutable);
    }

    // create mutable registers for tracking and editing data.
    let mut accelerations: [StepVec3D; N_BODIES] = core::array::from_fn(|_| StepVec3D::new()); // used to produce velocity changes
    let mut energies: [f64; N_BODIES] = [0.0; N_BODIES]; // used to check conservation.
    let mut last_total_energy: f64 = 0.0; // to check conservation/drift.

    let time_step_fp = SolarFp::from_f64(TIME_STEP)?; // used for velocities/positions, so SolarFp
    let half_time_step_fp = StepFp::from_f64(TIME_STEP / 2.0)?; // used for accelerations/velocities, so StepFp

    for step in 0..STEPS {
        if step % print_interval == 0 {
            // print data for current step.
            match print_type {
                PrintType::GraphSingle(p_index) => {
                    let pb = match bodies.get(p_index) {
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
                    for pb in bodies.iter() {
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

        // ALL BELOW CALCULATIONS ARE FOR THE NEXT STEP

        // calculate adjustments in velocity from n-body gravity
        #[allow(clippy::indexing_slicing)]
        // all indexing which occurs herein is *explicitly* bounded to the array length. Arrays are instantiated size N, and indexed with i.
        for i in 0..N_BODIES {
            // create iterator that reads all other planets.
            let (left, right) = bodies.split_at_mut(i);
            let (current, rest) = match right.split_first_mut() {
                Some(iter) => iter,
                None => unreachable!(
                    "None happens on empty list, bodies is guaranteed to be populated."
                ),
            };
            let others_iterator = left.iter().chain(rest.iter());

            // Calculate acceleration and gravitational potential from all other bodies
            let mut accel = StepVec3D::new();

            let mut gpe_accumulator = SolarFp::from_int(0); // stored as per kg of self, expanded later to full body.

            for (j, other) in others_iterator.enumerate() {
                let dir_vec = current.position.vector_to(&other.position);
                let distance = dir_vec.magnitude(); //fp60

                accel = accel.add(&calculate_accel(current, other));

                // only calculate potential against earlier planets; avoids double-counting
                if j < i {
                    gpe_accumulator -= other.gravity / distance;
                }
            }
            // calculate current energy
            let vel = current.velocity.magnitude().as_solar_fp();

            // this code deals in floating point arithmetic; deemed acceptable as it doesn't need perfect precision, and
            // is strictly used to validate that there are no long-term energy gains/losses.
            // this is also why we don't mind that the output is scaled by a planet's gravity rather than its mass.
            //                       GPE/kg             kinetic/kg
            let energy_per_kg = (gpe_accumulator + vel * vel / SolarFp::from_int(2)).to_f64();

            energies[i] = energy_per_kg * current.gravity.to_f64();

            // store acceleration for future calculations
            accelerations[i] = accel;
        }

        // apply adjustments in velocity/displacement with verlet method.
        #[allow(clippy::indexing_slicing)]
        // all indexing which occurs herein is *explicitly* bounded to the array length. Arrays are instantiated size N, and indexed with i.
        for i in 0..N_BODIES {
            let current = &mut bodies[i];
            let accel_first = accelerations[i];

            // verlet method
            println!("Applying step to: {}", current.name.to_ascii_upper());

            // apply half of acceleration-time to velocity
            let velocity_from_accel = accel_first.scale(half_time_step_fp);
            println!(
                "v_0:\t{:?};\nadding\t{:?}",
                current.velocity, velocity_from_accel
            );
            let temp_velocity = current.velocity.add(&velocity_from_accel);

            // apply effects of velocity on position
            let position_from_velocity = temp_velocity.as_solar().scale(time_step_fp);
            println!(
                "pos:\t{:?};\nadding\t{:?}",
                current.position, position_from_velocity
            );
            current.position = current.position.add(&position_from_velocity);

            // recalculate acceleration - as above
            // create iterator that reads all other planets.
            let (left, right) = bodies.split_at_mut(i);
            let (current, rest) = match right.split_first_mut() {
                Some(iter) => iter,
                None => unreachable!(
                    "None happens on empty list, bodies is guaranteed to be populated."
                ),
            };
            let others_iterator = left.iter().chain(rest.iter());
            let mut accel_second = StepVec3D::new();

            for other in others_iterator {
                accel_second = accel_second.add(&calculate_accel(current, other))
            }

            // add other half of acceleration-time to velocity with new accel.
            let second_velocity_from_accel = accel_second.scale(half_time_step_fp);
            println!(
                "v_0.5:\t{:?}\nadding\t{:?}\n",
                temp_velocity, second_velocity_from_accel
            );
            current.velocity = temp_velocity.add(&second_velocity_from_accel);

            // show result of computations
            println!(
                "final_pos: {:?}\nfinal_vel: {:?}\n",
                current.position, current.velocity
            );
        }

        // handle energy calculations.
        let new_sum: f64 = energies.iter().sum();

        if step > 0 && step % print_interval == 0 {
            println!(
                "System energy: {:.6e}\tchange: {:.6e} ({:+.2}%)",
                new_sum,
                new_sum - last_total_energy,
                (new_sum / last_total_energy - 1.0) * 100.0
            )
        }

        last_total_energy = new_sum;

        //
    }
    Ok(())
}

fn calculate_accel(pulled: &Body, pulling_body: &Body) -> StepVec3D {
    let v_to = pulled.position.vector_to(&pulling_body.position);
    let distance = v_to.magnitude();
    let direction_vector = v_to.to_unit_vector();

    #[cfg(test)]
    println!(
        "distance={:.4e}\ndir_vec={:?}",
        distance.to_f64(),
        direction_vector
    );

    let mut grav = pulling_body.gravity; // starts as GM, ends as GM/d^2
    let late_scaled: bool = pulling_body.id == 0 || pulled.parent_id == Some(pulling_body.id);

    if !late_scaled {
        // scale gravity immediately if its not going to be large (sun with any or gas giant with child)
        grav = grav.lshift(pulling_body.grav_scale);
    };

    #[cfg(test)]
    println!("gm={:.4e}", grav.to_f64());

    grav /= distance;

    #[cfg(test)]
    println!("gm/d={:.4e}", grav.to_f64());

    let mut grav = grav.as_step_fp(); // changes from SolarFp to StepFp here.

    #[cfg(test)]
    println!("gm/d_as_step={:.4e}", grav.to_f64());

    grav = grav.div_by_solar(&distance);

    #[cfg(test)]
    println!("gm/d2_unscaled={:.4e}", grav.to_f64());

    if late_scaled {
        // scale gravity now if it hasnt been done already.
        grav = grav.lshift(pulling_body.grav_scale);
    };

    #[cfg(test)]
    println!("gm/d2={:.4e}", grav.to_f64());

    // grav is now the magnitude of the gravitational force, the unit vector by it.
    direction_vector.scale_from_unit(grav)
}

#[test]
fn test_sun_earth_acceleration() {
    let b = BODIES;
    let sun = &b[0];

    let earth = &b[3];

    let accel = calculate_accel(earth, sun);

    // Expected: GM/r^2 ≈ 1.327e20 / (1.471e11)^2 ≈ 6.13e-3 m/s²
    // Direction should point from Earth toward Sun (roughly +x, -y given Earth's position)
    let ax = accel.0.to_f64();
    let ay = accel.1.to_f64();
    let az = accel.2.to_f64();
    let mag = (ax * ax + ay * ay + az * az).sqrt();

    let expected = 6.13e-3;
    let tolerance = 0.05e-3;
    assert!(
        (mag - expected).abs() < tolerance,
        "Acceleration magnitude {mag:.6e} not within tolerance of expected {expected:.6e}. Off by a factor of {:.2e}.", mag/expected
    );

    // Earth is at (-x, +y), so acceleration toward Sun should be (+x, -y)
    assert!(ax > 0.0, "Expected positive x-acceleration, got {ax:.6e}");
    assert!(ay < 0.0, "Expected negative y-acceleration, got {ay:.6e}");
}
