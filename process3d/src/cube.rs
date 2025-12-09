//! # Cube algorithms

use crate::{Aabb, Quad, Cube};
use crate::consume::Consumer;

/// Convert AABB to cube.
pub fn aabb_to_cube<T: Copy>((mi, ma): Aabb<T>) -> Cube<T> {
    [
        mi,
        [ma[0], mi[1], mi[2]],
        [mi[0], ma[1], mi[2]],
        [ma[0], ma[1], mi[2]],
        [mi[0], mi[1], ma[2]],
        [ma[0], mi[1], ma[2]],
        [mi[0], ma[1], ma[2]],
        ma,
    ]
}

/// Cube near quad.
pub fn cube_near(cube: &Cube) -> Quad {
    [cube[1], cube[0], cube[3], cube[2]]
}

/// Cube far quad.
pub fn cube_far(cube: &Cube) -> Quad {
    [cube[4], cube[5], cube[6], cube[7]]
}

/// Cube left quad.
pub fn cube_left(cube: &Cube) -> Quad {
    [cube[0], cube[4], cube[2], cube[6]]
}

/// Cube right quad.
pub fn cube_right(cube: &Cube) -> Quad {
    [cube[5], cube[1], cube[7], cube[3]]
}

/// Cube bottom quad.
pub fn cube_bottom(cube: &Cube) -> Quad {
    [cube[0], cube[1], cube[4], cube[5]]
}

/// Cube top quad.
pub fn cube_top(cube: &Cube) -> Quad {
    [cube[3], cube[2], cube[7], cube[6]]
}

/// Adds AABB cube to consumer.
pub fn add_cube_aabb<Scene, Material>(scene: &mut Scene, aabb: Aabb, mat: Material)
    where Scene: Consumer<(Aabb, Material)>, Material: Copy
{
    let f = scene.consumer();
    f(scene, (aabb, mat));
}
