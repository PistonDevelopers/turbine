//! Triangle algorithms.

use crate::{Aabb, Chunk, Plane, Triangle};
use crate::mask::CompressedMasks;
use crate::produce::*;

/// Calculates plane of triangle.
pub fn triangle_plane((a, b, c): Triangle) -> Plane {
    use vecmath::vec3_sub as sub;
    use vecmath::vec3_cross as cross;
    use vecmath::vec3_dot as dot;
    use vecmath::vec3_normalized as normalized;

    let e1 = sub(b, a);
    let e2 = sub(c, a);
    let n = normalized(cross(e1, e2));
    let d = dot(n, a);
    (n, d)
}

/// Triangle AABB.
pub fn triangle_aabb((a, b, c): Triangle) -> Aabb {
    let minx = a[0].min(b[0]).min(c[0]);
    let miny = a[1].min(b[1]).min(c[1]);
    let minz = a[2].min(b[2]).min(c[2]);
    let maxx = a[0].max(b[0]).max(c[0]);
    let maxy = a[1].max(b[1]).max(c[1]);
    let maxz = a[2].max(b[2]).max(c[2]);
    ([minx, miny, minz], [maxx, maxy, maxz])
}

/// Get triangle chunk from a slice of triangles with a mask.
///
/// The number of enabled bits in the mask tells the size of the triangle chunk.
#[inline(always)]
pub fn triangle_chunk<T>(list: &T, offset: usize) -> (Chunk<Triangle>, u64)
    where T: Produce<Triangle> + ?Sized
{
    (list.produce(offset), init_chunk_mask((&*list).virtual_length(), offset))
}

/// Enumerate triangle chunks in list according to a mask.
///
/// Skips the chunks which a zero mask.
///
/// Provides an offset index of the chunk.
pub fn chunk_iter<T: Produce<Triangle> + ?Sized>(
    list: &T,
    masks: &CompressedMasks
) -> impl Iterator<Item = (usize, (Chunk<Triangle>, u64))> + Clone {
    masks.iter().map(|(i, w)| {
        let i = i * 64;
        let (chunk, m) = triangle_chunk(list, i);
        (i, (chunk, m & w))
    }).filter(|(_, (_, w))| *w != 0)
}
