
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub use cam;

pub mod acc;
pub mod color;
pub mod consume;
pub mod cube;
pub mod fog;
pub mod frustrum;
pub mod mask;
pub mod math;
pub mod produce;
pub mod quad;
pub mod ray;
pub mod tile;
pub mod triangle;

/// Default prelude.
pub mod prelude {
    pub use crate::{
        *,
        acc::*,
        cam::*,
        color::*,
        consume::*,
        cube::*,
        fog::*,
        frustrum::*,
        math::*,
        produce::*,
        quad::*,
        ray::*,
        tile::*,
        triangle::*,
    };
}

/// RGB color.
pub type Rgb<T = f32> = [T; 3];
/// RGBA color.
pub type Rgba<T = f32> = [T; 4];
/// UV coordinate.
pub type Uv<T = f32> = [T; 2];
/// Circle.
pub type Circle<T = f32> = (Uv<T>, T);
/// Point.
pub type Point<T = f32> = [T; 3];
/// Vector.
pub type Vector<T = f32> = [T; 3];
/// Plane.
pub type Plane<T = f32> = (Point<T>, T);
/// Sphere.
pub type Sphere<T = f32> = (Point<T>, T);
/// Triangle.
pub type Triangle<T = f32> = (Point<T>, Point<T>, Point<T>);
/// Quad.
pub type Quad<T = f32> = [Point<T>; 4];
/// Cube.
pub type Cube<T = f32> = [Point<T>; 8];
/// Pixel position.
pub type PixelPos<T = u32> = [T; 2];
/// Tile position.
pub type TilePos<T = u32> = [T; 2];
/// Matrix transform.
pub type Matrix4<T = f32> = [[T; 4]; 4];
/// Line.
pub type Line<T = f32> = (Point<T>, Point<T>);
/// Ray.
pub type Ray<T = f32> = (Point<T>, Vector<T>);
/// Axis-Aligned Bounding Box.
pub type Aabb<T = f32> = (Point<T>, Point<T>);
/// Axis-Aligned Bounding Box for UV coordinates.
pub type UvAabb<T = f32> = (Uv<T>, Uv<T>);
/// Ray hit result.
pub type RayHit<T = f32> = Option<(T, usize)>;
/// Ray hit all result.
///
/// Uses an index flag, since the index is used to filter masks.
/// When the flag is true, it means the ray hit something new.
pub type RayHitAll<T = f32> = Option<(T, IndexFlag)>;
/// Standard chunk of 64 items.
///
/// This is designed to fit a 64 bit mask.
pub type Chunk<T> = [T; 64];
/// Standard consume function.
///
/// This is used to simplify processing over customized data structures.
pub type Consume<T, U> = fn(&mut T, U);

/// Used to store both an index and a flag in 64 bits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IndexFlag(pub i64);

impl IndexFlag {
    /// Get the index.
    #[inline(always)]
    pub fn index(&self) -> usize {
        if self.0 < 0 {(-(self.0 + 1)) as usize} else {self.0 as usize}
    }
    /// Get the flag.
    #[inline(always)]
    pub fn flag(&self) -> bool {self.0 < 0}
    /// Sets the flag to false.
    #[inline(always)]
    pub fn unflag(self) -> IndexFlag {IndexFlag::from_parts(self.index(), false)}
    /// Create index-flag from index and flag.
    #[inline(always)]
    pub fn from_parts(ind: usize, flag: bool) -> IndexFlag {
        IndexFlag(if flag {-(ind as i64) - 1} else {ind as i64})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask() {
        let mut masks = mask::CompressedMasks::new();
        masks.push(0);
        masks.push(0);
        masks.push(1);
        masks.push(0);
        masks.push(0);
        masks.push(0);
        masks.push(2);
        masks.push(3);
        masks.push(!0);
        masks.push(!0);

        let mut iter = masks.iter();
        assert_eq!(iter.next(), Some((2, 1)));
        assert_eq!(iter.next(), Some((6, 2)));
        assert_eq!(iter.next(), Some((7, 3)));
        assert_eq!(iter.next(), Some((8, !0)));
        assert_eq!(iter.next(), Some((9, !0)));
    }

    #[test]
    fn test_triangle_chunk() {
        let list: Vec<Triangle> = vec![([0.0; 3], [0.0; 3], [0.0; 3]); 72];
        let a = triangle::triangle_chunk(&list, 0);
        let b = triangle::triangle_chunk(&list[64..], 0);
        assert_eq!(a.1, 0xffffffffffffffff);
        assert_eq!(b.1, 0xff);
    }

    #[test]
    fn test_ray_triangle_chunk_hit_all_update() {
        use crate::ray::ray_triangle_chunk_hit_all_update;

        let mut hit = Some((0.0, IndexFlag::from_parts(0, false)));
        let eye = [0.0; 3];
        let dir = [1.0, 0.0, 0.0];
        let zero = ([0.0; 3], [0.0; 3], [0.0; 3]);
        ray_triangle_chunk_hit_all_update(
            (eye, dir),
            &[zero; 64],
            0,
            0,
            &mut hit,
        );
        assert_eq!(hit, Some((0.0, IndexFlag::from_parts(0, false))));
    }

    #[test]
    fn test_cube_faces_front() {
        use crate::prelude::*;

        let cube = aabb_to_cube(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));
        let (far_a, far_b) = quad_to_triangles(cube_far(&cube));
        assert!(triangle_point_front(far_a, [0.5, 0.5, 2.0]));
        assert!(triangle_point_front(far_b, [0.5, 0.5, 2.0]));
        let (near_a, near_b) = quad_to_triangles(cube_near(&cube));
        assert!(triangle_point_front(near_a, [0.5, 0.5, -1.0]));
        assert!(triangle_point_front(near_b, [0.5, 0.5, -1.0]));
        let (top_a, top_b) = quad_to_triangles(cube_top(&cube));
        assert!(triangle_point_front(top_a, [0.5, 2.0, 0.5]));
        assert!(triangle_point_front(top_b, [0.5, 2.0, 0.5]));
        let (bot_a, bot_b) = quad_to_triangles(cube_top(&cube));
        assert!(triangle_point_front(bot_a, [0.5, -1.0, 0.5]));
        assert!(triangle_point_front(bot_b, [0.5, -1.0, 0.5]));
        let (left_a, left_b) = quad_to_triangles(cube_left(&cube));
        assert!(triangle_point_front(left_a, [-1.0, 0.5, 0.5]));
        assert!(triangle_point_front(left_b, [-1.0, 0.5, 0.5]));
        let (right_a, right_b) = quad_to_triangles(cube_right(&cube));
        assert!(triangle_point_front(right_a, [2.0, 0.5, 0.5]));
        assert!(triangle_point_front(right_b, [2.0, 0.5, 0.5]));
    }

    #[test]
    fn test_cube_to_triangle() {
        use crate::produce::Produce;

        let far_a = ([0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]);
        let far_b = ([0.0, 1.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]);
        let near_a = ([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        let near_b = ([1.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let top_a = ([1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0]);
        let top_b = ([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]);
        let bot_a = ([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 1.0]);
        let bot_b = ([1.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let left_a = ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
        let left_b = ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]);
        let right_a = ([1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let right_b = ([1.0, 1.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        let zero = ([0.0; 3], [0.0; 3], [0.0; 3]);

        let data: &[Cube<u8>] = &[cube::aabb_to_cube(([0; 3], [1; 3]))];

        let chunk: Chunk<Triangle> = data.produce(0);
        assert_eq!(chunk[0], far_a);
        assert_eq!(chunk[1], far_b);
        assert_eq!(chunk[2], near_a);
        assert_eq!(chunk[3], near_b);
        assert_eq!(chunk[4], top_a);
        assert_eq!(chunk[5], top_b);
        assert_eq!(chunk[6], bot_a);
        assert_eq!(chunk[7], bot_b);
        assert_eq!(chunk[8], left_a);
        assert_eq!(chunk[9], left_b);
        assert_eq!(chunk[10], right_a);
        assert_eq!(chunk[11], right_b);
        assert_eq!(chunk[12], zero);

        let chunk: Chunk<Triangle> = data.produce(1);
        assert_eq!(chunk[0], far_b);
        assert_eq!(chunk[1], near_a);
        assert_eq!(chunk[2], near_b);
        assert_eq!(chunk[3], top_a);
        assert_eq!(chunk[4], top_b);
        assert_eq!(chunk[5], bot_a);
        assert_eq!(chunk[6], bot_b);
        assert_eq!(chunk[7], left_a);
        assert_eq!(chunk[8], left_b);
        assert_eq!(chunk[9], right_a);
        assert_eq!(chunk[10], right_b);
        assert_eq!(chunk[11], zero);

        let chunk: Chunk<Triangle> = data.produce(2);
        assert_eq!(chunk[0], near_a);
        assert_eq!(chunk[1], near_b);
        assert_eq!(chunk[2], top_a);
        assert_eq!(chunk[3], top_b);
        assert_eq!(chunk[4], bot_a);
        assert_eq!(chunk[5], bot_b);
        assert_eq!(chunk[6], left_a);
        assert_eq!(chunk[7], left_b);
        assert_eq!(chunk[8], right_a);
        assert_eq!(chunk[9], right_b);
        assert_eq!(chunk[10], zero);

        let chunk: Chunk<Triangle> = data.produce(3);
        assert_eq!(chunk[0], near_b);
        assert_eq!(chunk[1], top_a);
        assert_eq!(chunk[2], top_b);
        assert_eq!(chunk[3], bot_a);
        assert_eq!(chunk[4], bot_b);
        assert_eq!(chunk[5], left_a);
        assert_eq!(chunk[6], left_b);
        assert_eq!(chunk[7], right_a);
        assert_eq!(chunk[8], right_b);
        assert_eq!(chunk[9], zero);

        let chunk: Chunk<Triangle> = data.produce(4);
        assert_eq!(chunk[0], top_a);
        assert_eq!(chunk[1], top_b);
        assert_eq!(chunk[2], bot_a);
        assert_eq!(chunk[3], bot_b);
        assert_eq!(chunk[4], left_a);
        assert_eq!(chunk[5], left_b);
        assert_eq!(chunk[6], right_a);
        assert_eq!(chunk[7], right_b);
        assert_eq!(chunk[8], zero);

        let chunk: Chunk<Triangle> = data.produce(5);
        assert_eq!(chunk[0], top_b);
        assert_eq!(chunk[1], bot_a);
        assert_eq!(chunk[2], bot_b);
        assert_eq!(chunk[3], left_a);
        assert_eq!(chunk[4], left_b);
        assert_eq!(chunk[5], right_a);
        assert_eq!(chunk[6], right_b);
        assert_eq!(chunk[7], zero);

        let chunk: Chunk<Triangle> = data.produce(6);
        assert_eq!(chunk[0], bot_a);
        assert_eq!(chunk[1], bot_b);
        assert_eq!(chunk[2], left_a);
        assert_eq!(chunk[3], left_b);
        assert_eq!(chunk[4], right_a);
        assert_eq!(chunk[5], right_b);
        assert_eq!(chunk[6], zero);

        let chunk: Chunk<Triangle> = data.produce(7);
        assert_eq!(chunk[0], bot_b);
        assert_eq!(chunk[1], left_a);
        assert_eq!(chunk[2], left_b);
        assert_eq!(chunk[3], right_a);
        assert_eq!(chunk[4], right_b);
        assert_eq!(chunk[5], zero);

        let chunk: Chunk<Triangle> = data.produce(8);
        assert_eq!(chunk[0], left_a);
        assert_eq!(chunk[1], left_b);
        assert_eq!(chunk[2], right_a);
        assert_eq!(chunk[3], right_b);
        assert_eq!(chunk[4], zero);

        let chunk: Chunk<Triangle> = data.produce(9);
        assert_eq!(chunk[0], left_b);
        assert_eq!(chunk[1], right_a);
        assert_eq!(chunk[2], right_b);
        assert_eq!(chunk[3], zero);

        let chunk: Chunk<Triangle> = data.produce(10);
        assert_eq!(chunk[0], right_a);
        assert_eq!(chunk[1], right_b);
        assert_eq!(chunk[2], zero);

        let chunk: Chunk<Triangle> = data.produce(11);
        assert_eq!(chunk[0], right_b);
        assert_eq!(chunk[1], zero);

        let chunk: Chunk<Triangle> = data.produce(12);
        assert_eq!(chunk[0], zero);

        let data: &[Cube<u8>] = &[
            cube::aabb_to_cube(([0; 3], [1; 3])),
            cube::aabb_to_cube(([0; 3], [1; 3])),
        ];

        let chunk: Chunk<Triangle> = data.produce(0);
        assert_eq!(chunk[0], far_a);
        assert_eq!(chunk[1], far_b);
        assert_eq!(chunk[2], near_a);
        assert_eq!(chunk[3], near_b);
        assert_eq!(chunk[4], top_a);
        assert_eq!(chunk[5], top_b);
        assert_eq!(chunk[6], bot_a);
        assert_eq!(chunk[7], bot_b);
        assert_eq!(chunk[8], left_a);
        assert_eq!(chunk[9], left_b);
        assert_eq!(chunk[10], right_a);
        assert_eq!(chunk[11], right_b);
        assert_eq!(chunk[12], far_a);
        assert_eq!(chunk[13], far_b);
        assert_eq!(chunk[14], near_a);
        assert_eq!(chunk[15], near_b);
        assert_eq!(chunk[16], top_a);
        assert_eq!(chunk[17], top_b);
        assert_eq!(chunk[18], bot_a);
        assert_eq!(chunk[19], bot_b);
        assert_eq!(chunk[20], left_a);
        assert_eq!(chunk[21], left_b);
        assert_eq!(chunk[22], right_a);
        assert_eq!(chunk[23], right_b);
        assert_eq!(chunk[24], zero);

        let chunk: Chunk<Triangle> = data.produce(1);
        assert_eq!(chunk[0], far_b);
        assert_eq!(chunk[1], near_a);
        assert_eq!(chunk[2], near_b);
        assert_eq!(chunk[3], top_a);
        assert_eq!(chunk[4], top_b);
        assert_eq!(chunk[5], bot_a);
        assert_eq!(chunk[6], bot_b);
        assert_eq!(chunk[7], left_a);
        assert_eq!(chunk[8], left_b);
        assert_eq!(chunk[9], right_a);
        assert_eq!(chunk[10], right_b);
        assert_eq!(chunk[11], far_a);
        assert_eq!(chunk[12], far_b);
        assert_eq!(chunk[13], near_a);
        assert_eq!(chunk[14], near_b);
        assert_eq!(chunk[15], top_a);
        assert_eq!(chunk[16], top_b);
        assert_eq!(chunk[17], bot_a);
        assert_eq!(chunk[18], bot_b);
        assert_eq!(chunk[19], left_a);
        assert_eq!(chunk[20], left_b);
        assert_eq!(chunk[21], right_a);
        assert_eq!(chunk[22], right_b);
        assert_eq!(chunk[23], zero);

        let chunk: Chunk<Triangle> = data.produce(2);
        assert_eq!(chunk[0], near_a);
        assert_eq!(chunk[1], near_b);
        assert_eq!(chunk[2], top_a);
        assert_eq!(chunk[3], top_b);
        assert_eq!(chunk[4], bot_a);
        assert_eq!(chunk[5], bot_b);
        assert_eq!(chunk[6], left_a);
        assert_eq!(chunk[7], left_b);
        assert_eq!(chunk[8], right_a);
        assert_eq!(chunk[9], right_b);
        assert_eq!(chunk[10], far_a);
        assert_eq!(chunk[11], far_b);
        assert_eq!(chunk[12], near_a);
        assert_eq!(chunk[13], near_b);
        assert_eq!(chunk[14], top_a);
        assert_eq!(chunk[15], top_b);
        assert_eq!(chunk[16], bot_a);
        assert_eq!(chunk[17], bot_b);
        assert_eq!(chunk[18], left_a);
        assert_eq!(chunk[19], left_b);
        assert_eq!(chunk[20], right_a);
        assert_eq!(chunk[21], right_b);
        assert_eq!(chunk[22], zero);

        let chunk: Chunk<Triangle> = data.produce(3);
        assert_eq!(chunk[0], near_b);
        assert_eq!(chunk[1], top_a);
        assert_eq!(chunk[2], top_b);
        assert_eq!(chunk[3], bot_a);
        assert_eq!(chunk[4], bot_b);
        assert_eq!(chunk[5], left_a);
        assert_eq!(chunk[6], left_b);
        assert_eq!(chunk[7], right_a);
        assert_eq!(chunk[8], right_b);
        assert_eq!(chunk[9], far_a);
        assert_eq!(chunk[10], far_b);
        assert_eq!(chunk[11], near_a);
        assert_eq!(chunk[12], near_b);
        assert_eq!(chunk[13], top_a);
        assert_eq!(chunk[14], top_b);
        assert_eq!(chunk[15], bot_a);
        assert_eq!(chunk[16], bot_b);
        assert_eq!(chunk[17], left_a);
        assert_eq!(chunk[18], left_b);
        assert_eq!(chunk[19], right_a);
        assert_eq!(chunk[20], right_b);
        assert_eq!(chunk[21], zero);

        let chunk: Chunk<Triangle> = data.produce(4);
        assert_eq!(chunk[0], top_a);
        assert_eq!(chunk[1], top_b);
        assert_eq!(chunk[2], bot_a);
        assert_eq!(chunk[3], bot_b);
        assert_eq!(chunk[4], left_a);
        assert_eq!(chunk[5], left_b);
        assert_eq!(chunk[6], right_a);
        assert_eq!(chunk[7], right_b);
        assert_eq!(chunk[8], far_a);
        assert_eq!(chunk[9], far_b);
        assert_eq!(chunk[10], near_a);
        assert_eq!(chunk[11], near_b);
        assert_eq!(chunk[12], top_a);
        assert_eq!(chunk[13], top_b);
        assert_eq!(chunk[14], bot_a);
        assert_eq!(chunk[15], bot_b);
        assert_eq!(chunk[16], left_a);
        assert_eq!(chunk[17], left_b);
        assert_eq!(chunk[18], right_a);
        assert_eq!(chunk[19], right_b);
        assert_eq!(chunk[20], zero);

        let chunk: Chunk<Triangle> = data.produce(5);
        assert_eq!(chunk[0], top_b);
        assert_eq!(chunk[1], bot_a);
        assert_eq!(chunk[2], bot_b);
        assert_eq!(chunk[3], left_a);
        assert_eq!(chunk[4], left_b);
        assert_eq!(chunk[5], right_a);
        assert_eq!(chunk[6], right_b);
        assert_eq!(chunk[7], far_a);
        assert_eq!(chunk[8], far_b);
        assert_eq!(chunk[9], near_a);
        assert_eq!(chunk[10], near_b);
        assert_eq!(chunk[11], top_a);
        assert_eq!(chunk[12], top_b);
        assert_eq!(chunk[13], bot_a);
        assert_eq!(chunk[14], bot_b);
        assert_eq!(chunk[15], left_a);
        assert_eq!(chunk[16], left_b);
        assert_eq!(chunk[17], right_a);
        assert_eq!(chunk[18], right_b);
        assert_eq!(chunk[19], zero);

        let chunk: Chunk<Triangle> = data.produce(6);
        assert_eq!(chunk[0], bot_a);
        assert_eq!(chunk[1], bot_b);
        assert_eq!(chunk[2], left_a);
        assert_eq!(chunk[3], left_b);
        assert_eq!(chunk[4], right_a);
        assert_eq!(chunk[5], right_b);
        assert_eq!(chunk[6], far_a);
        assert_eq!(chunk[7], far_b);
        assert_eq!(chunk[8], near_a);
        assert_eq!(chunk[9], near_b);
        assert_eq!(chunk[10], top_a);
        assert_eq!(chunk[11], top_b);
        assert_eq!(chunk[12], bot_a);
        assert_eq!(chunk[13], bot_b);
        assert_eq!(chunk[14], left_a);
        assert_eq!(chunk[15], left_b);
        assert_eq!(chunk[16], right_a);
        assert_eq!(chunk[17], right_b);
        assert_eq!(chunk[18], zero);

        let chunk: Chunk<Triangle> = data.produce(7);
        assert_eq!(chunk[0], bot_b);
        assert_eq!(chunk[1], left_a);
        assert_eq!(chunk[2], left_b);
        assert_eq!(chunk[3], right_a);
        assert_eq!(chunk[4], right_b);
        assert_eq!(chunk[5], far_a);
        assert_eq!(chunk[6], far_b);
        assert_eq!(chunk[7], near_a);
        assert_eq!(chunk[8], near_b);
        assert_eq!(chunk[9], top_a);
        assert_eq!(chunk[10], top_b);
        assert_eq!(chunk[11], bot_a);
        assert_eq!(chunk[12], bot_b);
        assert_eq!(chunk[13], left_a);
        assert_eq!(chunk[14], left_b);
        assert_eq!(chunk[15], right_a);
        assert_eq!(chunk[16], right_b);
        assert_eq!(chunk[17], zero);

        let chunk: Chunk<Triangle> = data.produce(8);
        assert_eq!(chunk[0], left_a);
        assert_eq!(chunk[1], left_b);
        assert_eq!(chunk[2], right_a);
        assert_eq!(chunk[3], right_b);
        assert_eq!(chunk[4], far_a);
        assert_eq!(chunk[5], far_b);
        assert_eq!(chunk[6], near_a);
        assert_eq!(chunk[7], near_b);
        assert_eq!(chunk[8], top_a);
        assert_eq!(chunk[9], top_b);
        assert_eq!(chunk[10], bot_a);
        assert_eq!(chunk[11], bot_b);
        assert_eq!(chunk[12], left_a);
        assert_eq!(chunk[13], left_b);
        assert_eq!(chunk[14], right_a);
        assert_eq!(chunk[15], right_b);
        assert_eq!(chunk[16], zero);

        let chunk: Chunk<Triangle> = data.produce(9);
        assert_eq!(chunk[0], left_b);
        assert_eq!(chunk[1], right_a);
        assert_eq!(chunk[2], right_b);
        assert_eq!(chunk[3], far_a);
        assert_eq!(chunk[4], far_b);
        assert_eq!(chunk[5], near_a);
        assert_eq!(chunk[6], near_b);
        assert_eq!(chunk[7], top_a);
        assert_eq!(chunk[8], top_b);
        assert_eq!(chunk[9], bot_a);
        assert_eq!(chunk[10], bot_b);
        assert_eq!(chunk[11], left_a);
        assert_eq!(chunk[12], left_b);
        assert_eq!(chunk[13], right_a);
        assert_eq!(chunk[14], right_b);
        assert_eq!(chunk[15], zero);

        let chunk: Chunk<Triangle> = data.produce(10);
        assert_eq!(chunk[0], right_a);
        assert_eq!(chunk[1], right_b);
        assert_eq!(chunk[2], far_a);
        assert_eq!(chunk[3], far_b);
        assert_eq!(chunk[4], near_a);
        assert_eq!(chunk[5], near_b);
        assert_eq!(chunk[6], top_a);
        assert_eq!(chunk[7], top_b);
        assert_eq!(chunk[8], bot_a);
        assert_eq!(chunk[9], bot_b);
        assert_eq!(chunk[10], left_a);
        assert_eq!(chunk[11], left_b);
        assert_eq!(chunk[12], right_a);
        assert_eq!(chunk[13], right_b);
        assert_eq!(chunk[14], zero);

        let chunk: Chunk<Triangle> = data.produce(11);
        assert_eq!(chunk[0], right_b);
        assert_eq!(chunk[1], far_a);
        assert_eq!(chunk[2], far_b);
        assert_eq!(chunk[3], near_a);
        assert_eq!(chunk[4], near_b);
        assert_eq!(chunk[5], top_a);
        assert_eq!(chunk[6], top_b);
        assert_eq!(chunk[7], bot_a);
        assert_eq!(chunk[8], bot_b);
        assert_eq!(chunk[9], left_a);
        assert_eq!(chunk[10], left_b);
        assert_eq!(chunk[11], right_a);
        assert_eq!(chunk[12], right_b);
        assert_eq!(chunk[13], zero);

        let chunk: Chunk<Triangle> = data.produce(12);
        assert_eq!(chunk[0], far_a);
        assert_eq!(chunk[1], far_b);
        assert_eq!(chunk[2], near_a);
        assert_eq!(chunk[3], near_b);
        assert_eq!(chunk[4], top_a);
        assert_eq!(chunk[5], top_b);
        assert_eq!(chunk[6], bot_a);
        assert_eq!(chunk[7], bot_b);
        assert_eq!(chunk[8], left_a);
        assert_eq!(chunk[9], left_b);
        assert_eq!(chunk[10], right_a);
        assert_eq!(chunk[11], right_b);
        assert_eq!(chunk[12], zero);

        let chunk: Chunk<Triangle> = data.produce(13);
        assert_eq!(chunk[0], far_b);
        assert_eq!(chunk[1], near_a);
        assert_eq!(chunk[2], near_b);
        assert_eq!(chunk[3], top_a);
        assert_eq!(chunk[4], top_b);
        assert_eq!(chunk[5], bot_a);
        assert_eq!(chunk[6], bot_b);
        assert_eq!(chunk[7], left_a);
        assert_eq!(chunk[8], left_b);
        assert_eq!(chunk[9], right_a);
        assert_eq!(chunk[10], right_b);
        assert_eq!(chunk[11], zero);

        let chunk: Chunk<Triangle> = data.produce(14);
        assert_eq!(chunk[0], near_a);
        assert_eq!(chunk[1], near_b);
        assert_eq!(chunk[2], top_a);
        assert_eq!(chunk[3], top_b);
        assert_eq!(chunk[4], bot_a);
        assert_eq!(chunk[5], bot_b);
        assert_eq!(chunk[6], left_a);
        assert_eq!(chunk[7], left_b);
        assert_eq!(chunk[8], right_a);
        assert_eq!(chunk[9], right_b);
        assert_eq!(chunk[10], zero);

        let chunk: Chunk<Triangle> = data.produce(15);
        assert_eq!(chunk[0], near_b);
        assert_eq!(chunk[1], top_a);
        assert_eq!(chunk[2], top_b);
        assert_eq!(chunk[3], bot_a);
        assert_eq!(chunk[4], bot_b);
        assert_eq!(chunk[5], left_a);
        assert_eq!(chunk[6], left_b);
        assert_eq!(chunk[7], right_a);
        assert_eq!(chunk[8], right_b);
        assert_eq!(chunk[9], zero);
    }

    #[test]
    fn test_shift() {
        // println!("{:x}", 9223372036854775808_u64);
        // println!("{}", 0x80);
        // println!("{:b}", 0xfc);
        // println!("{:b}", 0xf8);
        assert_eq!(1_u64 << 63, 0x8000000000000000);
        let f = |ind: u64, off: u64| !((1_u64 << (ind - off)) - 1);
        assert_eq!(f(0, 0), 0xffffffffffffffff);
        assert_eq!(f(1, 0), 0xfffffffffffffffe);
        assert_eq!(f(2, 0), 0xfffffffffffffffc);
        assert_eq!(f(3, 0), 0xfffffffffffffff8);
    }
}
