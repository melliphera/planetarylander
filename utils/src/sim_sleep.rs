//! contains the sim_sleep function, allowing a thread to sleep for some time that corresponds to the input number of seconds passing in the simulation

use crate::StepFp;

pub fn sim_sleep(in_sim_time: StepFp, scale: StepFp) {
    let real_time = in_sim_time / scale;
    let duration = std::time::Duration::from_secs_f64(real_time.to_f64());
    std::thread::sleep(duration);
}
