//! Helper methods for math.

/// Vector type: x, y, z.
pub type Vec3 = [f32; 3];
/// Vector type: x, y, z, w.
pub type Vec4 = [f32; 4];
/// Matrix type.
pub type Mat4 = [[f32; 4]; 4];

use vecmath::{col_mat4_mul, col_mat4_transform};
use vecmath::{mat4_id, mat4_inv, mat4_transposed};
use vecmath::{vec3_add, vec3_dot, vec3_normalized, vec3_scale, vec3_sub};

/// Helper methods for vectors.
pub trait Vector {
    /// Returns a vector with zero length.
    fn zero() -> Self;
    /// Returns a vector in the forward direction.
    fn eye_forward() -> Self;
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
    /// Returns dot product of two vectors.
    fn dot(self, rhs: Vec3) -> f32;
    /// Returns the normalized vector.
    fn normalized(self) -> Self;
    /// Creates a homogeneous point.
    /// Puts 1.0 in the w component.
    fn point4(self) -> Vec4;
    /// Create a homogeneous vector.
    /// Puts 0.0 in the w component.
    fn vec4(self) -> Vec4;
    /// Cast to [i32; 2].
    fn i32x2(self) -> [i32; 2];
}

impl Vector for Vec3 {
    #[inline(always)]
    fn zero() -> Self {
        [0.0, 0.0, 0.0]
    }

    #[inline(always)]
    fn eye_forward() -> Self {
        [0.0, 0.0, 1.0]
    }

    #[inline(always)]
    fn from_2d(pos: [f64; 2], window_size: [u32; 2]) -> Self {
        let w = window_size[0] as f32;
        let h = window_size[1] as f32;
        [
            2.0 * (pos[0] as f32 - 0.5 * w) / w,
            -2.0 * (pos[1] as f32 - 0.5 * h) / h,
            0.0,
        ]
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
    fn dot(self, rhs: Self) -> f32 {
        vec3_dot(self, rhs)
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

    #[inline(always)]
    fn i32x2(self) -> [i32; 2] {
        [self[0] as i32, self[1] as i32]
    }
}

/// Helper methods for matrices.
pub trait Matrix {
    /// Returns identity matrix.
    fn id() -> Self;
    /// Returns transposed matrix, switching rows and columns.
    fn transposed(self) -> Self;
    /// Returns inverted matrix.
    fn inv(self) -> Self;
    /// Multiply with another matrix.
    fn mul(self, rhs: Self) -> Self;
    /// Transforms a vector in homogenous coordinates.
    fn transform(self, vec: Vec4) -> Vec4;
    /// Transform a point.
    fn pos(self, pos: Vec3) -> Vec3;
    /// Transform a vector.
    fn vec(self, vec: Vec3) -> Vec3;
    /// Transforms a ray through the matrix.
    fn ray(self, ray: Ray) -> Ray;
    /// Transforms a 3D point to frame buffer coordinates.
    /// Assumes that the matrix is model-view-projection.
    fn pos_to_frame_buffer(self, pos: Vec3, draw_size: [u32; 2]) -> Vec3;
}

impl Matrix for Mat4 {
    #[inline(always)]
    fn id() -> Self {
        mat4_id()
    }

    #[inline(always)]
    fn transposed(self) -> Self {
        mat4_transposed(self)
    }

    #[inline(always)]
    // fn inv(self) -> Self { mat4_inv(self) }
    fn inv(self) -> Self {
        use vecmath::{mat4_cast, Matrix4};

        let m: Matrix4<f64> = mat4_cast(self);
        mat4_cast(mat4_inv(m))
    }

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        col_mat4_mul(self, rhs)
    }

    #[inline(always)]
    fn transform(self, pos: Vec4) -> Vec4 {
        col_mat4_transform(self, pos)
    }

    #[inline(always)]
    fn pos(self, pos: Vec3) -> Vec3 {
        Vector::from_4d(self.transform(pos.point4()))
    }

    #[inline(always)]
    fn vec(self, vec: Vec3) -> Vec3 {
        Vector::from_4d(self.transform(vec.vec4()))
    }

    #[inline(always)]
    fn ray(self, ray: Ray) -> Ray {
        Ray {
            pos: self.pos(ray.pos),
            dir: self.vec(ray.dir).normalized(),
        }
    }

    #[inline(always)]
    fn pos_to_frame_buffer(self, pos: Vec3, draw_size: [u32; 2]) -> Vec3 {
        let pos = self.transform(pos.point4());
        [
            (pos[0] / pos[3] + 1.0) / 2.0 * draw_size[0] as f32,
            (draw_size[1] as f32 - (pos[1] / pos[3] + 1.0) / 2.0 * draw_size[1] as f32),
            0.0,
        ]
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

impl AABB {
    /// Returns empty AABB.
    pub fn empty() -> AABB {
        AABB {
            min: [0.0; 3],
            max: [0.0; 3],
        }
    }
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
    /// Creates a ray in view coordinates.
    pub fn from_2d(
        pos: [f64; 2],
        draw_size: [u32; 2],
        fov: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Ray {
        let pos: Vec3 = Vector::from_2d(pos, draw_size);
        let aspect_ratio = (draw_size[1] as f32) / (draw_size[0] as f32);
        let f = (fov * ::std::f32::consts::PI / 360.0).tan();
        let dx = pos[0] * f / aspect_ratio;
        let dy = pos[1] * f;
        let ray_near = [dx * near_clip, dy * near_clip, -near_clip];
        let ray_far = [dx * far_clip, dy * far_clip, -far_clip];
        Ray {
            pos: ray_near,
            dir: ray_far.sub(ray_near).normalized(),
        }
    }

    /// Returns position in ground plane.
    /// Returns `None` if looking up above the ground.
    /// Returns `None` if ground intersects the start position.
    pub fn ground_plane(&self) -> Option<Vec3> {
        let dy = self.dir[1];
        if dy.abs() < 0.000001 {
            None
        } else {
            let py = self.pos[1];
            let k = -py / dy;
            if k <= 0.0 {
                None
            } else {
                Some(self.pos.add(self.dir.scale(k)))
            }
        }
    }
}

/// Returns true when camera is looking in direction of a point.
pub fn is_looking_in_direction_of(camera_pos: Vec3, camera_forward: Vec3, point: Vec3) -> bool {
    let d = point.sub(camera_pos);
    d.dot(camera_forward) < 0.0
}
