# Apollo Guidance Computer Simulator

A historically-inspired spaceflight simulator implementing realistic orbital mechanics and safety-critical flight control systems. The project separates concerns between high-fidelity physics simulation and constrained embedded systems programming, mirroring the engineering challenges of the Apollo program.

[![Workspace CI](https://github.com/melliphera/planetarylander/actions/workflows/workspace.yml/badge.svg)](https://github.com/melliphera/planetarylander/actions/workflows/workspace.yml)

## Project status

This is a work in progress, and the design below describes where the project is heading.

- **Done:** the physics engine (`physics/`): an n-body simulation of the planets using a velocity Verlet integrator, with energy tracking, built on the custom fixed-point and vector types in `utils/`.
- **In progress:** the flight controller and sensor models (`rocket/`). Active development is on the [`feature/rocket-core`](https://github.com/melliphera/planetarylander/tree/feature/rocket-core) branch, which adds the solar system, sensor and controller threads with their message channels. It merges into `main` once it passes the strict MISRA-Rust lint gate, which denies unused code on `main`.
- **Next:** the controller's sensor handling: outlier rejection, null value handling and Kalman-style state estimation to keep it working when its inputs fail.

Design decisions and their reasoning are recorded in [`development_notes.md`](development_notes.md).

## Building and testing

Requires a stable Rust toolchain.

```sh
cargo test --workspace          # run every crate's unit tests
cargo clippy --workspace        # lints, including the MISRA-Rust rules on the rocket crate
cargo fmt --all -- --check      # formatting
```

Every push runs these checks in GitHub Actions. The rocket crate is additionally built in release mode with panicking operations (`unwrap`, `expect`, `panic!`, unchecked indexing) denied, per the MISRA-Rust ruleset in [`rocket/rocket_constraints.txt`](rocket/rocket_constraints.txt).

## Architecture

### Two-Tier Design

**Physics Engine (Conventional Rust)**
- N-body gravitational simulation using symplectic integrators
- Double-precision floating-point for energy conservation
- Implements realistic orbital mechanics for celestial bodies
- Provides ground truth for the spacecraft environment

**Flight Controller (MISRA-Rust)**
- Safety-critical code following MISRA-C principles adapted for Rust
- Operates under severe computational and reliability constraints
- Must maintain spacecraft control despite unreliable sensor data
- Models 1960s-era hardware limitations and failure modes

## Hardware Simulation

Each sensor runs on its own thread, modeling the unreliable nature of space-rated 1960s electronics:

### Failure Modes
- **Accuracy drift**: Readings degrade with environmental conditions (e.g., altimeter variance increases with altitude)
- **Hang/crash/reboot**: Modeled via `thread::sleep()`; controller must remain operational
- **Garbage data**: Valid data types containing physically impossible values requiring validation
- **Timing jitter**: ±5% variance in polling rates with occasional burst anomalies
- **Sensor drift**: Continuously growing offsets accumulate in readings

### Design Constraints
- One sensor per measurement (no redundancy for easy solutions)
- Flight controller polling must match instrument specification rates
- Limited "compute tokens" force selective data processing
- All decisions must be made with incomplete, suspect information

## Navigation Correction Systems

Two methods exist to counteract accumulated drift:

1. **Manual Star Sighting** (8/day maximum)
   - Duration: 15 minutes
   - Corrects: Position and orientation
   - Simulates human astronaut celestial navigation

2. **Mission Control Contact** (1/day maximum)
   - Round-trip signal time: ~2 minutes
   - Corrects: Position and velocity vectors
   - Models ground-based tracking systems

## MISRA-Rust Standards

Flight-critical code adheres to a Rust adaptation of MISRA-C guidelines, emphasizing:
- Deterministic behavior and bounded execution time
- Explicit error handling without panics
- Restricted use of dynamic allocation
- Prohibition of unsafe operations where possible
- Comprehensive input validation
- Defensive programming practices

*Note: MISRA-Rust is an inspired adaptation by the project author and has no affiliation with The MISRA Consortium.*

*Furthermore, the exact spec can be found in ./rocket/rocket_constraints.txt*

## Mission Profile

The simulation challenges the flight controller to:
- Maintain stable orbit despite accumulating sensor errors
- Detect and reject invalid telemetry data
- Manage computational budget across competing tasks
- Determine optimal timing for navigation corrections
- Operate safely through hardware failures and reboots

Success represents not just reaching a destination, but doing so with the reliability standards demanded of human-rated spacecraft systems.

## License

MIT