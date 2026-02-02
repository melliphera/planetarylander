pub mod orbit;
pub mod planets;
pub mod utils;

pub use orbit::simulate;

fn main() {
    simulate(utils::PrintType::GraphSingle(3), 2000);
}
