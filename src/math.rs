//! Helper methods for math.

/// Vector type.
pub type Vec3 = [f32; 3];
/// Matrix type.
pub type Mat4 = [[f32; 4]; 4];

use vecmath::{ vec3_add, vec3_sub, vec3_scale };
use vecmath::{ mat4_id };

/// Helper methods for vectors.
pub trait Vector {
    /// Adds two vectors.
    fn add(self, rhs: Self) -> Self;
    /// Subtracts two vectors.
    fn sub(self, rhs: Self) -> Self;
    /// Scales a vector.
    fn scale(self, f: f32) -> Self;
}

impl Vector for Vec3 {
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        vec3_add(self, rhs)
    }

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        vec3_sub(self, rhs)
    }

    #[inline(always)]
    fn scale(self, f: f32) -> Self {
        vec3_scale(self, f)
    }
}

/// Helper methods for matrices.
pub trait Matrix {
    /// Returns identity matrix.
    fn id() -> Self;
}

impl Matrix for Mat4 {
    #[inline(always)]
    fn id() -> Self { mat4_id() }
}
