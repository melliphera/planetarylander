//! Contains the Rocket struct. This serves as the interface between real data in the orbit (true position, velocity etc) and the faulty instruments, sensors and flight controller.
//! As such, these instruments will need to query Rocket for information from time to time; e.g. the altimeter needs to know the true distance to the surface in order to produce an unknown one.
//! Instruments must never store values acquired directly from Rocket without processing them to add their own inaccuracy first (this would be cheating!)

use agc_utils::{
    message_channels::RC, Quaternion, SimulationError, SolarFp, SolarVec3D, StepFp, StepVec3D,
    UnitFp,
};

use crate::{planets::Body, System};

const MAX_THRUST_ACCELERATION: StepFp = StepFp::from_int(60); // approx 6g, comparable to contemporary payload-bearing orbital rockets.

/// Representation of the physical model of the rocket.
#[allow(missing_docs)]
pub struct Rocket {
    pub position: SolarVec3D,
    pub velocity: StepVec3D,
    pub acceleration: StepVec3D,
    pub orientation: Quaternion,
    pub thrust: UnitFp,   // current thrust of the rocket on a scale of 0 to 1
    pub parent_id: usize, // the id of the closest/smallest body who's SOI the rocket is inside.
}

impl Rocket {
    pub fn new_on_earth() -> Self {
        //! spawns a new rocket; positioned on the Earth's equator on the far side of the sun, at epoch.
        Self {
            position: SolarVec3D::from_floats_trusted(
                -2.521202042891241E+10,
                1.449341963234861E+11,
                -6.164432685255142E+05,
            ),
            velocity: StepVec3D::from_floats_trusted(
                -3.025977219503563E+04,
                -5.280684209504152E+03,
                3.173286860330564E+01,
            ),
            acceleration: StepVec3D::from_floats_trusted(0.0, 0.0, 0.0),
            orientation: match Quaternion::from_floats(
                0.643668685702801,
                0.0,
                3.255040401211003E-06,
                0.765304268271799,
            ) {
                Ok(q) => q,
                Err(_) => unreachable!("Quaternion has been precomputed."),
            },
            thrust: UnitFp::from_int(0),
            parent_id: 3, // starts on earth.
        }
    }

    pub fn new_100km_above_earth() -> Self {
        //! spawns a new rocket; positioned 100km above the Earth's equator on the far side of the sun, at epoch.
        let mut r = Self::new_on_earth();
        r.position = SolarVec3D::from_floats_trusted(
            -2.521202214251241E+10,
            1.449342948654861E+11,
            -6.164432727165142E+05,
        );
        r.velocity = r
            .velocity
            .add(&StepVec3D::from_floats_trusted(8161.0, 1400.0, -31.0));
        r
    }

    pub const fn process_command(&mut self, command: RC) {
        //! modifies the rocket's internal state to respect the command sent.
        match command {
            RC::Thrust(val) => self.thrust = val,
        }
    }

    pub fn calculate_accel(&mut self, sys: &System) {
        //! calculate the acceleration of the rocket thanks to its own effects and gravity from the bodies in System.
        let thrust_accel_mag = MAX_THRUST_ACCELERATION.scale_by_unit(self.thrust);

        // total acceleration vector. Instantiating term is the thrust vector.
        let mut accel_vec = self
            .orientation
            .to_forward_vector()
            .scale_from_unit(thrust_accel_mag);
        for body in sys.bodies.iter() {
            accel_vec = accel_vec.add(&self.accel_from_body(body))
        }
        self.acceleration = accel_vec
    }

    fn accel_from_body(&self, body: &Body) -> StepVec3D {
        //! Calculates the gravitational acceleration applied to the rocket by a single body.
        let v_to = self.position.vector_to(&body.position);
        let distance = v_to.magnitude();
        let direction_vector = v_to.to_unit_vector();

        // to widen the bounds on intermediate numbers used in this process, all maths is done on i128 versions of the original numbers.
        // this value below is the total amount of left-shift required for solarFp_internal / solarFp^2_internal to be used as the internal for StepFp
        // while keeping the same value. Under current values, the greatest this can be is 66 (when dealing with the Sun's gravity).
        // therefore as long as distance is greater than 4m, 2-step left shifting will always be enough.
        let mut shift_needed: u32 = (agc_utils::fixed_point::STEP_FIXED_POINT_DECIMAL_BITS
            + agc_utils::fixed_point::SOLAR_FIXED_POINT_DECIMAL_BITS
            + body.gravity.scale) as u32;

        // shift gravity as far left as we can. Doing as much as possible as early as possible maximises accuracy.
        let mut grav_int = body.gravity.stored_solar.internal_value() as i128; // starts as GM, ends as GM/d^2
        let to_shift = grav_int.leading_zeros().min(shift_needed); // shift as much as needed, but bounded by register size. Leftover gets tidied in 2nd pass.
        grav_int <<= to_shift;
        shift_needed -= to_shift;

        // grab distance internal value.
        let dist_int = distance.internal_value() as i128;

        // do first gravity divide.
        grav_int /= dist_int;

        // clear up remaining shift if there is any. Again doing asap for best accuracy.
        grav_int <<= shift_needed;

        // do second gravity divide
        grav_int /= dist_int;

        // wrap into StepFp, ensuring it will fit.
        assert!(grav_int.leading_zeros() >= 64);
        let grav = StepFp::with_internal(grav_int as i64);

        // debug prints to check acceleration.
        //println!("{}: {:.4e}m/s^2", &body.name.to_ascii_upper()[..3], grav.to_f64());

        // grav is now the magnitude of the gravitational force, scale the unit vector by it.
        direction_vector.scale_from_unit(grav)
    }

    pub(crate) fn verlet_1(&mut self, sys: &System, time: f64) -> Result<(), SimulationError> {
        let half_time = StepFp::from_f64(time / 2.0)?;
        let full_time = SolarFp::from_f64(time)?;

        self.calculate_accel(sys);

        // add half of acceleration-time to velocity and use that to work out new position.
        self.velocity = self.velocity.add(&self.acceleration.scale(half_time));
        self.position = self
            .position
            .add(&self.velocity.as_solar().scale(full_time));
        Ok(())
    }

    pub(crate) fn verlet_2(&mut self, sys: &System, time: f64) -> Result<(), SimulationError> {
        let half_time = StepFp::from_f64(time / 2.0)?;

        // recalculate acceleration from new position and use it to get new velocity.
        self.calculate_accel(sys);
        self.velocity = self.velocity.add(&self.acceleration.scale(half_time));

        Ok(())
    }
}
