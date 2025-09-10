//! Frustrum algorithms.

use crate::{Aabb, Chunk, Plane, Point, RayHit, Triangle, Uv};
use crate::triangle::{triangle_aabb, triangle_plane};
use cam::CameraPerspective;

/// Frustum planes.
#[derive(Copy, Clone, Debug)]
pub struct FrustumPlanes<T = f32> {
    /// The near plane.
    pub near: Plane<T>,
    /// The far plane.
    pub far: Plane<T>,
    /// The left plane.
    pub left: Plane<T>,
    /// The right plane.
    pub right: Plane<T>,
    /// The top plane.
    pub top: Plane<T>,
    /// The bottom plane.
    pub bottom: Plane<T>,
}

/// Get the near plane of camera perspective.
///
/// The normal of this plane points away from the camera origin.
pub fn near_plane(persp: &CameraPerspective) -> Plane {
    ([0.0, 0.0, 1.0], -persp.near_clip)
}

/// Get the far plane of camerea perspective.
///
/// The normal of this plane points toward the camera origin.
pub fn far_plane(persp: &CameraPerspective) -> Plane {
    ([0.0, 0.0, -1.0], persp.far_clip)
}

/// Get dimensions of near clip plane.
pub fn near_dim(persp: &CameraPerspective) -> Uv {
    use vecmath::traits::Radians;

    let fov = persp.fov.deg_to_rad();
    let near_height = 2.0 * persp.near_clip * (fov / 2.0).tan();
    let near_width = near_height * persp.aspect_ratio;
    [near_width, near_height]
}

/// Get the coordinate of the UV in near clip plane.
pub fn near_uv_pos(persp: &CameraPerspective, dim: Uv, uv: Uv) -> Point {
    [0.5 * dim[0] * uv[0], 0.5 * dim[1] * uv[1], persp.near_clip]
}

/// Calculates left plane using UV x-coordinate.
///
/// The normal of this plane points toward the right plane.
pub fn left_plane(persp: &CameraPerspective, dim: Uv, uv_x: f32) -> Plane {
    let eye = [0.0; 3];
    let top = near_uv_pos(persp, dim, [uv_x, -1.0]);
    let bottom = near_uv_pos(persp, dim, [uv_x, 1.0]);
    triangle_plane((eye, bottom, top))
}

/// Calculates right plane using UV x-coordinate.
///
/// The normal of this plane points toward the right plane.
pub fn right_plane(persp: &CameraPerspective, dim: Uv, uv_x: f32) -> Plane {
    let eye = [0.0; 3];
    let top = near_uv_pos(persp, dim, [uv_x, -1.0]);
    let bottom = near_uv_pos(persp, dim, [uv_x, 1.0]);
    triangle_plane((eye, top, bottom))
}

/// Calculates the top plane using UV y-coordinate.
///
/// The normal of this plane points toward the bottom plane.
pub fn top_plane(persp: &CameraPerspective, dim: Uv, uv_y: f32) -> Plane {
    let eye = [0.0; 3];
    let left = near_uv_pos(persp, dim, [-1.0, uv_y]);
    let right = near_uv_pos(persp, dim, [1.0, uv_y]);
    triangle_plane((eye, right, left))
}

/// Calculates the bottom plane using UV y-coordinate.
///
/// The normal of this plane points toward the top plane.
pub fn bottom_plane(persp: &CameraPerspective, dim: Uv, uv_y: f32) -> Plane {
    let eye = [0.0; 3];
    let left = near_uv_pos(persp, dim, [-1.0, uv_y]);
    let right = near_uv_pos(persp, dim, [1.0, uv_y]);
    triangle_plane((eye, left, right))
}

/// Frustum planes for a tile.
pub fn frustum_planes_tile(persp: &CameraPerspective, dim: Uv, tile_pos: Uv, tile_size: Uv) -> FrustumPlanes {
    FrustumPlanes {
        near: near_plane(persp),
        far: far_plane(persp),
        left: left_plane(persp, dim, tile_pos[0]),
        right: right_plane(persp, dim, tile_pos[0] + tile_size[0]),
        top: top_plane(persp, dim, tile_pos[1] + tile_size[1]),
        bottom: bottom_plane(persp, dim, tile_pos[1]),
    }
}

/// Returns `true` if point is at the front of plane.
pub fn plane_point_front((n, d): Plane, p: Point) -> bool {
    use vecmath::vec3_dot as dot;

    let d2 = dot(n, p) + d;
    d2 >= 0.0
}

/// Returns `true` if any corner of an AABB is in front of plane.
pub fn plane_aabb_front_or_intersect(p: Plane, (mi, ma): Aabb) -> bool {
    plane_point_front(p, mi) ||
    plane_point_front(p, [ma[0], mi[1], mi[2]]) ||
    plane_point_front(p, [mi[0], ma[1], mi[2]]) ||
    plane_point_front(p, [ma[0], ma[1], mi[2]]) ||
    plane_point_front(p, [mi[0], mi[1], ma[2]]) ||
    plane_point_front(p, [ma[0], mi[1], ma[2]]) ||
    plane_point_front(p, [mi[0], ma[1], ma[2]]) ||
    plane_point_front(p, ma)
}

/// Returns `true` if AABB intersects frustrum planes.
pub fn frustum_planes_aabb_intersect(fr: &FrustumPlanes, a: Aabb) -> bool {
    plane_aabb_front_or_intersect(fr.near, a) &&
    plane_aabb_front_or_intersect(fr.far, a) &&
    plane_aabb_front_or_intersect(fr.left, a) &&
    plane_aabb_front_or_intersect(fr.right, a) &&
    plane_aabb_front_or_intersect(fr.top, a) &&
    plane_aabb_front_or_intersect(fr.bottom, a)
}

/// Generate bit mask for triangle chunk where AABB intersects frustum planes.
///
/// Uses existing bits to avoid processing triangles that are not needed.
pub fn frustum_planes_triangle_chunk_mask(
    fr: &FrustumPlanes,
    chunk: &Chunk<Triangle>,
    bits: u64
) -> u64 {
    if bits == 0 {return 0};

    let mut mask: u64 = 0;
    for i in 0..64 {
        if (bits >> i) & 1 != 1 {continue};

        if frustum_planes_aabb_intersect(fr, triangle_aabb(chunk[i])) {
            mask |= 1 << i;
        }
    }
    mask
}

/// Linear transformation of depth using near and far clip distance.
pub fn depth_linear(persp: &CameraPerspective, depth: RayHit) -> f32 {
    if let Some((t, _)) = depth {
        (t - persp.near_clip) / (persp.far_clip - persp.near_clip)
    } else {
        1.0
    }
}
