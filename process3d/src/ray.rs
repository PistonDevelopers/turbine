//! # Ray algorithms

use crate::{Chunk, IndexFlag, PixelPos, Point, Ray, RayHit, RayHitAll, Triangle};
use crate::frustrum::{near_dim, near_uv_pos};
use crate::cam::CameraPerspective;

/// Converts `RayHitAll` to `RayHit`.
pub fn ray_hit_all_to_ray_hit(hit: RayHitAll) -> RayHit {
    if let Some((depth, index_flag)) = hit {
        Some((depth, index_flag.index()))
    } else {None}
}

/// Ray triangle intersection using Möller-Trumbore algorithm.
pub fn ray_triangle_hit((origin, direction): Ray, (a, b, c): Triangle) -> Option<f32> {
    use vecmath::vec3_sub as sub;
    use vecmath::vec3_cross as cross;
    use vecmath::vec3_dot as dot;

    let e1 = sub(b, a);
    let e2 = sub(c, a);

    let ray_cross_e2 = cross(direction, e2);
    let det = dot(e1, ray_cross_e2);

    // Check whether ray is parallel to this triangle.
    let eps = f32::EPSILON;
    if det > -eps && det < eps {return None}

    let inv_det = 1.0 / det;
    let s = sub(origin, a);
    let u = inv_det * dot(s, ray_cross_e2);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_cross_e1 = cross(s, e1);
    let v = inv_det * dot(direction, s_cross_e1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = inv_det * dot(e2, s_cross_e1);

    if t > eps { // ray intersection
        return Some(t);
    }

    None
}

/// Ray intersection against a triangle chunk with a mask.
pub fn ray_triangle_chunk_hit(
    ray: Ray,
    chunk: &Chunk<Triangle>,
    mask: u64
) -> RayHit {
    if mask == 0 {return None};

    let mut min: Option<(f32, usize)> = None;
    for i in 0..64 {
        if (mask >> i) & 1 == 1 {
            if let Some(t) = ray_triangle_hit(ray, chunk[i]) {
                if min.is_none() || t < min.unwrap().0 {
                    min = Some((t, i))
                }
            }
        }
    }
    min
}

/// Ray intersection against all triangles in chunk with a mask.
///
/// This is used when rendering semi-transparent objects.
/// The mask is used to filter out previous objects.
pub fn ray_triangle_chunk_hit_all(
    ray: Ray,
    chunk: &Chunk<Triangle>,
    mask: u64,
) -> RayHit {
    if mask == 0 {return None};

    for i in 0..64 {
        if (mask >> i) & 1 == 1 {
            if let Some(t) = ray_triangle_hit(ray, chunk[i]) {
                return Some((t, i));
            }
        }
    }
    None
}

/// Offset ray hit index.
pub fn ray_hit_offset(hit: RayHit, off: usize) -> RayHit {
    if let Some((d, i)) = hit {
        Some((d, i + off))
    } else {None}
}

/// Ray hit of triangle chunk with mask, updating hit.
pub fn ray_triangle_chunk_hit_update(
    ray: Ray,
    chunk: &Chunk<Triangle>,
    mask: u64,
    off: usize,
    res: &mut RayHit,
) {
    *res = match (*res, ray_hit_offset(ray_triangle_chunk_hit(ray, &chunk, mask), off)) {
        (None, x) | (x, None) => x,
        (Some((ti, mi)), Some((tj, mj))) => {
            if tj < ti {Some((tj, mj))} else {Some((ti, mi))}
        }
    }
}

/// Ray hit of all triangles in chunk with mask, updating hit.
///
/// A new mask is pre-computed to filter out previous hits in the chunk.
pub fn ray_triangle_chunk_hit_all_update(
    ray: Ray,
    chunk: &Chunk<Triangle>,
    mask: u64,
    off: usize,
    res: &mut RayHitAll,
) {
    let mask = if let Some((_, ind)) = *res {
        let ind = if ind.flag() {return} else {ind.index()};
        if ind >= off + 64 {return} else if ind >= off {
            !((1_u64 << (ind - off)) - 1) & mask
        } else {mask}
    } else {return};
    *res = match (*res, ray_hit_offset(ray_triangle_chunk_hit_all(ray, &chunk, mask), off)) {
        (x, None) => x,
        (_, Some((tj, mj))) => Some((tj, IndexFlag::from_parts(mj, true))),
    }
}

/// Ray hit of iterator over triangle chunks.
///
/// This can be used when you only need to check one ray per triangle chunk.
pub fn ray_triangle_chunk_iter_hit(
    ray: Ray,
    iter: impl Iterator<Item = (usize, (Chunk<Triangle>, u64))>
) -> RayHit {
    let mut min: Option<(f32, usize)> = None;
    for (off, (chunk, mask)) in iter {
        ray_triangle_chunk_hit_update(ray, &chunk, mask, off, &mut min);
    }
    min
}

/// Calculate ray direction.
pub fn ray_dir(
    persp: &CameraPerspective,
    eye: Point,
    pos: PixelPos,
    dim: PixelPos,
) -> Point {
    use vecmath::vec3_sub as sub;
    use vecmath::vec3_normalized as normalized;

    let x = (pos[0] as f32 + 0.5) / dim[0] as f32 * 2.0 - 1.0;
    let y = (pos[1] as f32 + 0.5) / dim[1] as f32 * 2.0 - 1.0;
    let ndim = near_dim(persp);
    let npos = near_uv_pos(persp, [x, y], ndim);
    normalized(sub(npos, eye))
}
