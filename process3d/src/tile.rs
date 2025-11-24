//! Tile rendering algorithms.

use crate::{PixelPos, Point, RayHit, RayHitAll, TilePos, Triangle, Uv};
use crate::cam::CameraPerspective;
use crate::frustrum::{
    frustum_planes_tile,
    frustum_planes_triangle_chunk_mask,
    near_dim,
};
use crate::mask::CompressedMasks;
use crate::ray::{ray_dir, ray_triangle_chunk_hit_update, ray_triangle_chunk_hit_all_update};
use crate::triangle::{chunk_iter, triangle_chunk};
use crate::produce::Produce;

/// From camera perspective, tile and a list of triangles, get mask of intersecting triangles.
///
/// This is used as a preparation stage before sampling each tile in parallel.
///
/// The algorithm does not clear the masks before pushing new ones.
pub fn tile_mask<T: Produce<Triangle> + ?Sized>(
    persp: &CameraPerspective,
    dim: Uv,
    tile_pos: Uv,
    tile_size: Uv,
    list: &T,
    masks: &mut CompressedMasks,
) {
    let fr = frustum_planes_tile(persp, dim, tile_pos, tile_size);
    let mut i = 0;
    let n = list.virtual_length();
    loop {
        if i >= n {break}
        let (chunk, bits) = triangle_chunk(list, i);
        masks.push(frustum_planes_triangle_chunk_mask(&fr, &chunk, bits));
        i += 64;
    }
}

/// Calculate the normalized tile position.
///
/// Dimension is the size of image in pixels.
///
/// Indices are in the grid of tiles.
///
/// Tile size is both width and height in pixels.
pub fn tile_pos(dim: PixelPos, [i, j]: TilePos, tile_size: u32) -> Uv {
    let x = (i as f32 * tile_size as f32) / dim[0] as f32 * 2.0 - 1.0;
    let y = (j as f32 * tile_size as f32) / dim[1] as f32 * 2.0 - 1.0;
    [x, y]
}

/// Calculate the normalized tile size.
///
/// Dimension is the size of image in pixels.
///
/// Indices are in the grid of tiles.
///
/// Tile size is both width and height in pixels.
pub fn tile_size(dim: PixelPos, [i, j]: TilePos, tile_size: u32) -> Uv {
    let tw = (dim[0] as f32 / tile_size as f32).ceil() * tile_size as f32;
    let th = (dim[1] as f32 / tile_size as f32).ceil() * tile_size as f32;
    let mix = i as f32 * tile_size as f32;
    let miy = j as f32 * tile_size as f32;
    let max = (i + 1) as f32 * tile_size as f32;
    let may = (j + 1) as f32 * tile_size as f32;
    [
        (max.min(tw) - mix) / dim[0] as f32 * 2.0,
        (may.min(th) - miy) / dim[1] as f32 * 2.0,
    ]
}

/// Calculate the grid size of tiles needed to cover image.
///
/// Dimension is the size of image in pixels.
///
/// Indices are in the grid of tiles.
pub fn tile_grid(dim: PixelPos, tile_size: u32) -> [u32; 2] {
    let tw = (dim[0] as f32 / tile_size as f32).ceil();
    let th = (dim[1] as f32 / tile_size as f32).ceil();
    [tw as u32, th as u32]
}

/// Prepare compressed masks.
pub fn pre_masks(
    dim: PixelPos,
    n_tile_size: u32,
) -> Vec<CompressedMasks> {
    let [w, h] = tile_grid(dim, n_tile_size);
    let n = w * h;
    vec![CompressedMasks::new(); n as usize]
}

/// Fake masks that includes all triangles.
///
/// This is used for testing.
pub fn fake_all_masks<T: Produce<Triangle> + ?Sized + Sync>(
    _persp: &CameraPerspective,
    _dim: PixelPos,
    _n_tile_size: u32,
    list: &T,
    masks: &mut [CompressedMasks]
) {
    for masks in masks {
        masks.clear();
        masks.push_ones(list.virtual_length() as u64);
    }
}

/// Collect all masks per tile.
pub fn masks<T: Produce<Triangle> + ?Sized + Sync>(
    persp: &CameraPerspective,
    dim: PixelPos,
    n_tile_size: u32,
    list: &T,
    masks: &mut [CompressedMasks]
) {
    use rayon::prelude::*;

    let w = tile_grid(dim, n_tile_size)[0];
    let ndim = near_dim(&persp);
    masks.par_iter_mut().enumerate().for_each(|(k,  masks)| {
        masks.clear();
        let i = k as u32 % w;
        let j = k as u32 / w;
        let tpos = tile_pos(dim, [i, j], n_tile_size);
        let tsize = tile_size(dim, [i, j], n_tile_size);
        tile_mask(persp, ndim, tpos, tsize, list, masks);
    });
}

/// Render depth of a tile using a camera perspective, image resolution,
/// tile position, tile size and triangle list with mask, into a tile depth and index buffer.
///
/// Iterates through triangle chunks and updates the tile depth and index buffer.
///
/// Ray direction is recreated for each triangle chunk.
///
/// Requires compressed masks per tile to be prepared in advance.
pub fn render_tile_depth<T: Produce<Triangle> + ?Sized>(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    n_tile_size: u32,
    list: &T,
    masks: &CompressedMasks,
    tile: &mut [RayHit],
) {
    let eye = [0.0; 3];
    let iter = chunk_iter(list, masks);
    for (off, (chunk, mask)) in iter {
        for j in 0..n_tile_size {
            for i in 0..n_tile_size {
                let dir: Point = ray_dir(persp, eye, [pos[0] + i, pos[1] + j], dim);
                ray_triangle_chunk_hit_update((eye, dir), &chunk, mask, off,
                    &mut tile[(j * n_tile_size + i) as usize]);
            }
        }
    }
}

/// Render depth of a tile using a camera perspective, image resolution,
/// tile position, tile size and triangle list with mask, into a tile depth and index buffer.
///
/// This can be used to render semi-transparent objects,
/// because the ray visits all triangles.
///
/// Notice that there is no guaranteed order.
/// This has to be managed either by pre-ordering or by post-processing.
///
/// `RayHitAll` in `tile` should be initialized to `Some((0.0, IndexFlag::from_parts(0, false)))`.
/// When `None`, the ray will not progress further.
/// Check `IndexFlag::flag` to see whether the ray hit something new.
///
/// Iterates through all triangles in chunks and updates the tile depth and index buffer.
///
/// Ray direction is recreated for each triangle chunk.
///
/// Requires compressed masks per tile to be prepared in advance.
///
/// Returns `true` if there is something to render.
/// You can use a loop and break when this is `false`.
pub fn render_tile_depth_all<T: Produce<Triangle> + ?Sized>(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    n_tile_size: u32,
    list: &T,
    masks: &CompressedMasks,
    tile: &mut [RayHitAll],
) -> bool {
    use crate::IndexFlag;

    let eye = [0.0; 3];
    let iter = chunk_iter(list, masks);
    let mut alive = false;
    for (off, (chunk, mask)) in iter {
        alive = false;
        for j in 0..n_tile_size {
            for i in 0..n_tile_size {
                let hit = &mut tile[(j * n_tile_size + i) as usize];
                let dir: Point = ray_dir(persp, eye, [pos[0] + i, pos[1] + j], dim);
                ray_triangle_chunk_hit_all_update((eye, dir), &chunk, mask, off, hit);
                if let Some((d, index_flag)) = hit {
                    if !index_flag.flag() {
                        let ind = index_flag.index();
                        let new_ind = ind.max(off + 64);
                        *hit = Some((*d, IndexFlag::from_parts(new_ind, false)));
                    }
                }
                alive |= hit.is_some();
            }
        }
        // Skip iteration if there no rays alive or nothing to render.
        if !alive {return false}
    }

    // Terminate rays when not hitting anything new.
    for hit in tile {
        if let Some((_, index_flag)) = hit {
            if !index_flag.flag() {*hit = None};
        }
    }

    alive
}
