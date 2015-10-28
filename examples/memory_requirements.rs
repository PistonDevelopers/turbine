extern crate turbine;

use turbine::*;

fn main() {
    use std::mem::size_of;

    let world = size_of::<World>();
    // init, prev, current, next.
    let physics = 4 * size_of::<[Vec3; world::ENTITY_COUNT]>();
    let sum = world + physics;
    println!("Memory requirements {} MiB", sum as f64 / 1024.0 / 1024.0);
}
