
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub use cam;

pub mod color;
pub mod consume;
pub mod cube;
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
        cam::*,
        color::*,
        consume::*,
        cube::*,
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
}
