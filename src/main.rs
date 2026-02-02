#!allow(dead_code)]

mod orbit;
mod planets;
mod utils;

mod rocket;

use orbit::simulate;

fn main() {
    simulate(utils::PrintType::GraphSingle(3), 2000);
}
