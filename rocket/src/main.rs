use agc_physics::{simulate, utils};

fn main() {
    simulate(utils::PrintType::GraphSingle(3), 2000);
}
