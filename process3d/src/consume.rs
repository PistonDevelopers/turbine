//! Consume Pattern.
//!
//! The Consume Pattern is used for generic programming of processing algorithms.
//! It lifts 3D processing algorithms into as many possible compatible data structure targets.
//!
//! This pattern also avoids the need to implement traits for scene data structures.
//! Usually, you can just process data with `Vec` first, before adding it to the scene
//! using a normal method on the scene data structure, e.g. for triangles or quads.
//!
//! This means that you can process 3D data in the format that takes less memory and provides
//! the highest abstraction that you want to use.
//!
//! For more information, see the [Consumer] trait.

use crate::{Aabb, Consume, Cube, Point, Triangle, Quad};

/// Implemented by data structures that consume something.
///
/// This is used to simplify processing over customized data structures.
///
/// - A `Vec<T>` type consumes `T`
///
/// For all `Material` types that implement the `Copy` trait:
///
/// - Any consumer of `(Triangle, Material)` consumes `(Quad, Material)`
/// - Any consumer of `(Quad, Material)` consumes `(Cube, Material)`
/// - Any consumer of `(Cube, Material)` consumes `(Aabb, Material)` (of cubes)
/// - Any consumer of `(Aabb, Material)` consumes `(Point, Material)` (of unit cubes)
///
/// This is also used to simplify voxel processing:
/// For any integer types `T` of `u8, u16, u32, u64, i8, i16, i32, i64`,
/// any consumer of `(Point, Material)` consumes `(Point<T>, Material)`.
///
/// As a result, a simple `Vec<(Triangle, Material>` can be used in many processing algorithms.
/// However, you can choose to use e.g. `Vec<(Quad, Material)>` instead,
/// which will work with algorithms based on quads or higher, but not for triangles directly.
///
/// You can use the `consume_all` method to process from an iterator.
/// This method picks the proper consumer function automatically
/// that is compatible with the consumer.
///
/// Hint: You can use palette indices for materials, or a pointer reference to implement the `Copy` trait.
pub trait Consumer<T> {
    /// Returns consume function.
    fn consumer(&self) -> Consume<Self, T>;

    /// Consume all the data from some iterator.
    fn consume_all(&mut self, data: impl Iterator<Item = T>) {
        let f = self.consumer();
        for d in data {f(self, d)}
    }
}

impl<T> Consumer<T> for Vec<T> {
    fn consumer(&self) -> Consume<Self, T> {Vec::push}
}

impl<T, Material> Consumer<(Quad, Material)> for T
    where T: Consumer<(Triangle, Material)>, Material: Copy
{
    fn consumer(&self) -> Consume<Self, (Quad, Material)> {
        |scene, (quad, mat)| {
            let f = scene.consumer();
            let (a, b) = crate::quad::quad_to_triangles(quad);
            f(scene, (a, mat));
            f(scene, (b, mat));
        }
    }
}

impl<T, Material> Consumer<(Cube, Material)> for T
    where T: Consumer<(Quad, Material)>, Material: Copy
{
    fn consumer(&self) -> Consume<Self, (Cube, Material)> {
        use crate::cube::*;
        |scene, (cube, mat)| {
            let f = scene.consumer();
            let quads = [
                cube_far(&cube),
                cube_near(&cube),
                cube_top(&cube),
                cube_bottom(&cube),
                cube_left(&cube),
                cube_right(&cube),
            ];
            for q in quads {f(scene, (q, mat))}
        }
    }
}

impl<T, Material> Consumer<(Aabb, Material)> for T
    where T: Consumer<(Cube, Material)>
{
    fn consumer(&self) -> Consume<Self, (Aabb, Material)> {
        |scene, (aabb, mat)| {
            let f = scene.consumer();
            let c = crate::cube::aabb_to_cube(aabb);
            f(scene, (c, mat));
        }
    }
}

impl<T, Material> Consumer<(Point, Material)> for T
    where T: Consumer<(Aabb, Material)>, Material: Copy
{
    fn consumer(&self) -> Consume<Self, (Point, Material)> {
        use vecmath::vec3_add;
        |scene, (pt, mat)| {
            let f = scene.consumer();
            f(scene, ((pt, vec3_add(pt, [1.0; 3])), mat));
        }
    }
}

macro_rules! integer {
    ($($t:ty),*) => {$(
        impl<T, Material> Consumer<(Point<$t>, Material)> for T
            where T: Consumer<(Point, Material)>, Material: Copy
        {
            fn consumer(&self) -> Consume<Self, (Point<$t>, Material)> {
                |scene, (pt, mat)| {
                    let f = scene.consumer();
                    f(scene, ([pt[0] as f32, pt[1] as f32, pt[2] as f32], mat));
                }
            }
        }
    )*}
}

integer!{u8, u16, u32, u64, i8, i16, i32, i64}
