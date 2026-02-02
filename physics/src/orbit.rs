//! This file is responsible for the time-step simulation to produce orbital motion.
use crate::planets::{BODIES, N_BODIES};
use crate::utils::{PrintType, Vec3D};

const TIME_STEP: f64 = 432.0; // 200 steps per day
const SIM_TIME: f64 = 86400.0 * 365.25 * 2.0; // 2 earth years
const STEPS: usize = (SIM_TIME / TIME_STEP) as usize;

const BIG_G: f64 = 6.6743015e-11;

pub fn simulate(print_type: PrintType, print_interval: usize) {
    let mut bodies = BODIES; // copy for mutability
    for body in bodies.iter_mut() {
        body.fill_influencers(&BODIES);
    };

    // create mutable registers for tracking and editing data.
    let mut accelerations: [Vec3D; N_BODIES] = core::array::from_fn(|_| Vec3D::new()); // used to produce velocity changes
    let mut energies: [f64; N_BODIES] = [0.0; N_BODIES];   // used to check conservation.
    let mut last_total_energy: f64 = 0.0; // to check conservation/drift.

    for step in 0..STEPS {
        if step % print_interval == 0 {
            // print data for current step.
            match print_type {
                PrintType::GraphSingle(p_index) => {
                    let pb = &bodies[p_index];
                    println!("{}, {}, {}, {}", pb.name[..3].to_uppercase(), pb.position.0, pb.position.1, pb.position.2)
                }
                PrintType::GraphAll => {
                    for pb in bodies.iter() {
                        println!("{}, {}, {}, {}", pb.name[..3].to_uppercase(), pb.position.0, pb.position.1, pb.position.2)
                    }
                }
            }
        }




        // ALL BELOW CALCULATIONS ARE FOR THE NEXT STEP

        // calculate adjustments in velocity from n-body gravity
        for i in 0..N_BODIES {
            // create iterator that reads all other planets.
            let (left, right) = bodies.split_at_mut(i);
            let (current, rest) = right.split_first_mut().unwrap();
            let others_iterator = left.iter().chain(rest.iter());

            // Calculate acceleration and gravitational potential from all other bodies
            let mut accel = Vec3D(0.0, 0.0, 0.0);
            let mut current_potential = 0f64; // stored as per kg of self, expanded later to full body.

            for (j, other) in others_iterator.enumerate() { 
                let dir_vec = current.position.vector_to(&other.position);
                let distance = dir_vec.magnitude();
                accel = accel.add(&dir_vec.scale(other.gravity / (distance.powi(3))));
                // only calculate potential against earlier planets; avoids double-counting
                if j < i {
                    current_potential -= other.gravity/distance;
                }
            }
            // calculate current energy
            let vel = current.velocity.magnitude();

            //               GPE/kg             kinetic/kg             kg
            energies[i] = (current_potential + vel * vel/2.0) * (current.gravity/BIG_G);

            // store acceleration for future calculations
            accelerations[i] = accel;
        }

        // apply adjustments in velocity/displacement with verlet method.
        for i in 0..N_BODIES {
            let current = &mut bodies[i];
            let accel_first = accelerations[i];

            // verlet method

            // apply half of acceleration-time to velocity
            let temp_velocity = current.velocity.add(&accel_first.scale(TIME_STEP / 2.0));

            // apply effects of velocity on position
            current.position = current.position.add(&temp_velocity.scale(TIME_STEP));

            // recalculate acceleration - as above
            // create iterator that reads all other planets.
            let (left, right) = bodies.split_at_mut(i);
            let (current, rest) = right.split_first_mut().unwrap();
            let others_iterator = left.iter().chain(rest.iter());
            let mut accel_second = Vec3D(0.0, 0.0, 0.0);

            for other in others_iterator { 
                let dir_vec = current.position.vector_to(&other.position);
                let distance = dir_vec.magnitude();
                accel_second = accel_second.add(&dir_vec.scale(other.gravity / (distance.powi(3))));
            }

            // add other half of acceleration-time to velocity with new accel.
            current.velocity = temp_velocity.add(&accel_second.scale(TIME_STEP / 2.0));
        }

        // handle energy calculations.
        let new_sum: f64 = energies.iter().sum();

        if step > 0 && step % print_interval == 0 {
            println!("System energy: {:.6e}\t change: {:.6e} ({:+.2}%)", new_sum, new_sum-last_total_energy, (new_sum/last_total_energy-1.0)*100.0)
        }

        last_total_energy = new_sum;

        //
    }
}
