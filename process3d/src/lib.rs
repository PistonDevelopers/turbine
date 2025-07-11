
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod mask;

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
}
