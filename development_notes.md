# Development Journey
I figured it would be pertinent to keep notes on the development experience over the course of this project, discussing why certain decisions have been made, problems that have been run into and the solutions I chose to implement for them. This document intends to be a chronological record of these things. The purpose is not to get too heavily into the technical, but explore overarching decisions. It should be noted that over the course of development, this will be a living document. I intend not to delete anything, and discuss certain parts both before and after writing them. As such, the writing tense may jump all over the place.

**Unlike README.md, this document's entire purpose is for personal development and reflection on the design process. As such, 0% of it is written with AI.**

# n-body Orbital Physics
This was a fun one to do. It strongly represents my typical development experience in Rust, both in terms of the style used and type of work. The only particular differences in the workflow here is learning to use Clippy linting and cargo fmt; in my personal projects I have a somewhat opinionated formatting style, but this was intended to align with production code in an environment where I am not the one to be making those decisions.

In terms of technical challenges, they were somewhat limited here too. One issue I found during my initial simulations, which used naive Euler iteration, was an unbounded growth in the energy of the system of approximately 7% per Earth year. Rocketry and mechanical systems in general are not an area in which I have any expertise whatsoever. However after consulting the internet, I learned about the Verlet method which, while not removing inaccuracy, keeps the system energy oscillating within a tight bound of the true values, making it suitable for these kind of systems where eradicating drift is much more important than short-term accuracy. 

# The Rocket
This is the part that I really intended to challenge my abilities. This is working under lots of constraints and using many methods with which I was generally unfamiliar with at the start of the project. Further details of these constraints can be seen in src/rocket_constraints.txt, but the intention is to make fast and lightweight code that works within the most stringent standards as are used for systems in aerospace, vehicles, medical settings and any other situation where lives depend on the code running cleanly. To further complicate things, I've made the effort to minimise the use of other libraries as convenience tools. Exceptions to this are arrayvec (I've implemented similar in a private personal project before) and tokio's watch - I explicitly want to use channel-based concurrency but std::sync::mpsc wasn't capable of overwritable non-growing send buffers.

## Code Skeleton
Step 1 was to produce a filesystem for this project that I was happy with, with some of the basic linking code and shared struct definitions done. The Flight Controller forms the "brain" of the rocket, and as such will have an extensive amount of logic associated with it. This logic is going to manage the rocket at a very granular level, specifically reading data from sensors, sending instructions to the rocket engine and reactor wheels et cetera based on its own trajectory calculations. As such this is going to be very broad, and have its own top-level folder, seperate from the FlightController definition. Sensors have their own folder, but their (in-world) runtime logic is pretty simple so will be contained with the definitions. 

## Altimeter - First Pass
Once this was all in place, the next step was to start implementing the hardware simulation logic. The altimeter was going to be the first piece as it seemed pretty simple - pass it a target body's position and its own position, work out the true distance, and then apply hardware errors to this value. However during the testing of this, it came to my attention that one of the harshest Clippy lints (mandated by the MISRA-RUST standard) disallows any floating-point arithmetic. This started a dive into writing some of the primitives we'll be using throughout this project, something which in hindsight, I really should have set out to do before looking at any specific tooling. The extra-problematic thing about this is that via git's pre-commit hooks, these linting issues prohibit me from making commmits. This means the next commit doesn't happen for a *very* long time.

## Primitives
### FixedPoint<N>
#### Implementation
The first primitive I looked at was the FixedPoint<N> struct; a basic numeric type which stores an i64 and uses it to perform fractional arithmetic. The N in question defines the number of digits which represent the fractional part, such that using 0-indexed bits, the Nth bit represents unity. Thanks to the presence of the sign bit too, this leaves 63-N bits for integer values.

This type can handle all the basic mathematical operations; addition, subtraction, multiplication, division, negation and absolution. They can be instantiated either from i64 (converted; this doesn't just inject the i64 as the internal representation) or from f64.

#### N selection
It is intended to use as few different values of N as possible over the course of this, as conversions between different Ns inherently lose accuracy, leaving us with the very same problems we were trying to avoid by avoiding floating point arithmetic.

As such, three N values are proposed for this project:

1) **FixedPoint<60> aka UnitFp; range -8<x<8 with 10^-17 precision**. This one is extraordinarily accurate for sub-integer operations; in fact, thanks to its decimal bit width being 8 bits larger than the f64 mantissa (though in a sense, 7 bits thanks to the implicit leading 1 in f64), it's 128x more accurate than f64 for values greater than 0.5, 64x more accurate for values over 0.25 etc; the precision is only lesser at values below 0.001953125 and even then the difference is marginal.

2) **FixedPoint<40> aka StepFp; max abs value ~8e6, 10^-12 precision**. This one is used during per-step calculations that require a higher degree of accuracy than SolarFp below. For example, the gravitational acceleration between Saturn and Neptune at their greatest distance is in the region of 10^-6 m/s^2. While this seems small enough to ignore, it is exactly these types of minute perturbation that 

3) **FixedPoint<20> aka SolarFp; max abs value ~8e12, 1/1048576 precision**. This one provides a high level of accuracy while working on scales of distance within the Solar System; 

#### Issues in development
Talk about bounds checking, magnitude, float conversion

**Sanity Validation**

Despite them being banned in the final product, use of f64s for testing against the limitations of the FP system was extremely helpful. While it's easy to validate that the actual numbers used fit within the bounds of the FP spec, intermediate values produced during calculation were far harder to validate. In order to do this, I created a Vec3D_f64 in my utils crate, and in testing, exported a type alias: 

```pub type Vec3D = Vec3D_f64```

 This allowed me to swap this testing struct in wherever the usual Vec3D should be used without modifying any code at its use points. This struct shadowed the API of the normal Vec3D exactly, but with extra bounds checks added to each method; namely if the limit of the SolarFP was exceeded, it would print a warning to stdout. As it turns out, the vectors themselves fit within the bounds of the SolarFp at all times; a much welcome result. 

**Magnitude**

Calculating vector magnitudes proved to be a significant test of the limitations of the FP type. Given that the first step of calculating magnitudes is squaring the components, this couldn't be done natively within the FP spec as the numbers would shoot out of bounds near-instantly; as such a specific new method for squaring an FP had to be implemented. For this, we get to jump into a little bit of mathematical logic:

- Squaring a number doubles its bit length.
- Squaring a FixedPoint<N> is equivalent to squaring its internal number (a) and doubling its number of decimal bits <N>. More formally:
    - A FixedPoint<N>(a) called 'x' can be represented mathematically as x = a*2^(-N)
    - Therefore x^2 = a*a * 2^(-N)*2^(-N) = a^2 * 2^(-2N)
    - And to represent this again as a FixedPoint, x^2 = FixedPoint<2N>(a^2)
    
- By definition, the square root does the inverse, so sqrt(x) = FixedPoint<N/2>(sqrt(a))

- Addition and subtraction of 2 values with the same N does not modify N, i.e.
    - FixedPoint<N>(a) + FixedPoint<N>(b) = FixedPoint<N>(a+b)

For the magnitude operation, we have square -> add -> square root.
By the rules above, for our N value, the process is double -> do nothing -> halve, therefore N goes to N and can be ignored.
For our 'a' values, the process is square (double bits, make positive) -> add -> square root (halve bits); the same operations as would be done on any other numeric type. Perfect!

'a' consists of 63 value bits and 1 sign bit. Squaring it always makes it positive, and doubles the number of bits it takes up.
Therefore a^2 can be represented as a 126 bit unsigned integer.

For addition a+b=c, the size of the result c can never be more than 1 bit greater than the largest of a or b. 
More generally, addition of N values adds at most ceiling(log_2(N)) bits to the greatest individual value.
Therefore the addition of up to 4 different squared i64 is guaranteed to fit in a u128.
Therefore to calculate magnitude of a 3-FP vector the following is guaranteed to work:

 1) take the internal values of the 3 FP's, .abs() them, and convert to u128.
 2) Square the 3 u128s and add them together.
 3) Assert this new number has at least 2 leading zeros (within the scales used, it definitely will).
 4) Square-root the new u128 and convert to i64; the assertion above ensures it will fit despite the sign bit.
 return FP(ans) where ans is the result of 4) and the <N> value of FP is the same as the initial 3.

**Scaling**

While doing the mental validation of what would be an acceptable fixed point range, I used the distance to Pluto as my upper bound guideline, I wasn't expecting values to extend much beyond that. However when I started converting the codebase to use my two defined FixedPoints, it became clear to me that there are some other numbers that sit well outside that. I had stored "gravity" as a field on my solar objects rather than mass, thinking that this would cut down the masses to sensible levels; after all, 10^11 is a pretty significant divisor. However the gravity of the sun still sat 3 orders of magnitude outside what my SolarFp could handle. 

This was eventually solved with body factors. GM values continue to be stored as f64, but then particularly large values have special handling when theyre being used in the gravitational calculations. The first step of the acceleration calculation now chooses an adjusted GM based on the circumstances, and takes note of the scale factor as follows:

If the pulling body is the Sun, GM is divided by 1048576; a number chosen for its proximity to 1 million, while being a power of 2 so division (and the later multiplication) are fast processes.

If the pulling body is a gas giant (Jupiter, Saturn, Uranus, Neptune) and is the pulled body's parent (ie pulled body is a moon or rocket within the gas giant's Hill sphere), GM is divided by 1024, similar logic as above but close to 1000.