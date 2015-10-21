//! Helper methods for math.

/// Vector type: x, y, z.
pub type Vec3 = [f32; 3];
/// Vector type: x, y, z, w.
pub type Vec4 = [f32; 4];
/// Matrix type.
pub type Mat4 = [[f32; 4]; 4];

use vecmath::{ vec3_add, vec3_sub, vec3_scale, vec3_normalized };
use vecmath::{ mat4_id, mat4_inv };

/// Helper methods for vectors.
pub trait Vector {
    /// Returns a vector with zero length.
    fn zero() -> Self;
    /// Returns a vector in the forward direction.
    fn forward() -> Self;
    /// Creates vector from 2D position.
    /// Returns a vector in normalized coordinates.
    fn from_2d(pos: [f64; 2], window_size: [u32; 2]) -> Self;
    /// Creates vector from 4D position.
    /// Ignores the w component.
    fn from_4d(pos: [f32; 4]) -> Self;
    /// Adds two vectors.
    fn add(self, rhs: Self) -> Self;
    /// Subtracts two vectors.
    fn sub(self, rhs: Self) -> Self;
    /// Scales a vector.
    fn scale(self, f: f32) -> Self;
    /// Returns the normalized vector.
    fn normalized(self) -> Self;
    /// Creates a homogeneous point.
    /// Puts 1.0 in the w component.
    fn point4(self) -> Vec4;
    /// Create a homogeneous vector.
    /// Puts 0.0 in the w component.
    fn vec4(self) -> Vec4;
}

impl Vector for Vec3 {
    #[inline(always)]
    fn zero() -> Self { [0.0, 0.0, 0.0] }

    #[inline(always)]
    fn forward() -> Self { [0.0, 0.0, 1.0] }

    #[inline(always)]
    fn from_2d(pos: [f64; 2], window_size: [u32; 2]) -> Self {
        let w = window_size[0] as f32;
        let h = window_size[1] as f32;
        [2.0 * (pos[0] as f32 - 0.5 * w) / w,
        -2.0 * (pos[1] as f32 - 0.5 * h) / h, 0.0]
    }

    #[inline(always)]
    fn from_4d(pos: [f32; 4]) -> Self {
        [pos[0], pos[1], pos[2]]
    }

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

    #[inline(always)]
    fn normalized(self) -> Self {
        vec3_normalized(self)
    }

    #[inline(always)]
    fn point4(self) -> Vec4 {
        [self[0], self[1], self[2], 1.0]
    }

    #[inline(always)]
    fn vec4(self) -> Vec4 {
        [self[0], self[1], self[2], 0.0]
    }
}

/// Helper methods for matrices.
pub trait Matrix {
    /// Returns identity matrix.
    fn id() -> Self;
    /// Returns inverted matrix.
    fn inv(self) -> Self;
    /// Transforms a ray through the matrix.
    fn ray(self, ray: Ray) -> Ray;
}

impl Matrix for Mat4 {
    #[inline(always)]
    fn id() -> Self { mat4_id() }

    #[inline(always)]
    fn inv(self) -> Self { mat4_inv(self) }

    #[inline(always)]
    fn ray(self, ray: Ray) -> Ray {
        use vecmath::col_mat4_transform;

        Ray {
            pos: Vector::from_4d(
                col_mat4_transform(self, ray.pos.point4())),
            dir: vec3_normalized(Vector::from_4d(
                col_mat4_transform(self, ray.dir.vec4()))),
        }
    }
}

/// An AABB rectangle.
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    /// The corner with lowest coordinates.
    pub min: Vec3,
    /// The corner with highest coordinates.
    pub max: Vec3,
}

/// A ray.
#[derive(Debug, Copy, Clone)]
pub struct Ray {
    /// The position of the ray.
    pub pos: Vec3,
    /// The direction of the ray.
    pub dir: Vec3,
}

impl Ray {
    /// Returns position in ground plane.
    /// Returns `None` if looking up above the ground.
    /// Returns `None` if looking down below the ground.
    /// Returns `None` if ground intersects the start position.
    pub fn ground_plane(&self) -> Option<Vec3> {
        let dy = self.dir[1];
        if dy.abs() < 0.000001 { None }
        else {
            let py = self.pos[1];
            let k = -py / dy;
            if k <= 0.0 { None }
            else { Some(self.pos.add(self.dir.scale(k))) }
        }
    }
}
