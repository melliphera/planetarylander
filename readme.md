# Apollo Guidance Computer Simulator

A historically-inspired spaceflight simulator implementing realistic orbital mechanics and safety-critical flight control systems. The project separates concerns between high-fidelity physics simulation and constrained embedded systems programming, mirroring the engineering challenges of the Apollo program.

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