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

/// Get optimal tile size for sub-tiling.
///
/// When sub-tiling is enabled, it should override the tile size set by the user,
/// to use a better number with more divisors.
pub const fn optimal_sub_tile_size(tile_size: u32) -> u32 {
    match tile_size {
        0..=12 => 12,
        13..=24 => 24,
        25..=36 => 36,
        37..=48 => 48,
        49..=60 => 60,
        _ => 120,
    }
}

/// Calculate the largest divisor sub-tile-size where average number of triangles
/// is less than `k`.
///
/// The input of `opt_tile_size` should be an output of `optimal_sub_tile_size`.
///
/// The initial condition of sub-tiling is `triangles_in_tile >= k`.
pub const fn sub_tile(opt_tile_size: u32, triangles_in_tile: u32, k: u32) -> u32 {
    match opt_tile_size {
        12 => {
            // Do binary search on `2, 3, 4, 6`.
            let n = opt_tile_size / 4;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 3;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 2}
                else {return 3}
            };

            let n = opt_tile_size / 6;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 4}
            else {return 6}
        }
        24 => {
            // Do binary search on `2 3 4 6 8 12`.
            let n = opt_tile_size / 6;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 3;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 2};

                let n = opt_tile_size / 4;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 3}
                else {return 4}
            }

            let n = opt_tile_size / 8;
            let y = opt_tile_size / (n * n);
            if y >= k {return 6};

            let n = opt_tile_size / 12;
            let y = opt_tile_size / (n * n);
            if y >= k {return 8}
            else {return 12}
        }
        36 => {
            // Do binary search on `2 3 4 6 9 12 18`.
            let n = opt_tile_size / 6;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 3;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 2}

                let n = opt_tile_size / 4;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 3}
                else {return 4}
            }

            let n = opt_tile_size / 12;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 9;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 6}
                else {return 9}
            }

            let n = opt_tile_size / 18;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 12}
            else {return 18}
        }
        48 => {
            // Do binary search on `2 3 4 6 8 12 16 24`.
            let n = opt_tile_size / 8;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 4;
                let y = triangles_in_tile / (n * n);
                if y >= k {
                    let n = opt_tile_size / 3;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {return 2}
                    else {return 3}
                }

                let n = opt_tile_size / 6;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 4}
                else {return 6}
            }

            let n = opt_tile_size / 16;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 12;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 8}
                else {return 12}
            }

            let n = opt_tile_size / 24;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 16}
            else {return 24}
        }
        60 => {
            // Do binary search on `2 3 4 5 6 10 12 15 20 30`.
            let n = opt_tile_size / 10;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 4;
                let y = triangles_in_tile / (n * n);
                if y >= k {
                    let n = opt_tile_size / 3;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {return 2}
                    else {return 3}
                }

                let n = opt_tile_size / 6;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 5}
                else {return 6}
            }

            let n = opt_tile_size / 15;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 12}

            let n = opt_tile_size / 20;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 15}

            let n = opt_tile_size / 30;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 20}
            else {return 30}
        }
        _ => {
            // Do binary search on `2 3 4 5 6 8 10 12 15 20 24 30 40 60`.
            let n = opt_tile_size / 12;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 6;
                let y = triangles_in_tile / (n * n);
                if y >= k {
                    let n = opt_tile_size / 4;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {
                        let n = opt_tile_size / 3;
                        let y = triangles_in_tile / (n * n);
                        if y >= k {return 2}
                        else {return 3}
                    }

                    let n = opt_tile_size / 5;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {return 4}
                    else {return 5}
                }

                let n = opt_tile_size / 10;
                let y = triangles_in_tile / (n * n);
                if y >= k {
                    let n = opt_tile_size / 8;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {return 6}
                    else {return 8}
                } else {return 10}
            }

            let n = opt_tile_size / 24;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 20;
                let y = triangles_in_tile / (n * n);
                if y >= k {
                    let n = opt_tile_size / 15;
                    let y = triangles_in_tile / (n * n);
                    if y >= k {return 12}
                    else {return 15}
                } else {return 20}
            }

            let n = opt_tile_size / 40;
            let y = triangles_in_tile / (n * n);
            if y >= k {
                let n = opt_tile_size / 30;
                let y = triangles_in_tile / (n * n);
                if y >= k {return 24}
                else {return 30}
            }

            let n = opt_tile_size / 60;
            let y = triangles_in_tile / (n * n);
            if y >= k {return 40}
            else {return 60}
        }
    }
}

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

/// From camera perspective, tile and a list of triangles, get mask of intersecting triangles.
///
/// This is used as a preparation stage before sampling each tile in parallel.
///
/// The algorithm does not clear the masks before pushing new ones.
pub fn tile_mask_with_pre_mask<T: Produce<Triangle> + ?Sized>(
    persp: &CameraPerspective,
    dim: Uv,
    tile_pos: Uv,
    tile_size: Uv,
    list: &T,
    masks: &mut CompressedMasks,
    pre_masks: &CompressedMasks,
) {
    let fr = frustum_planes_tile(persp, dim, tile_pos, tile_size);
    let iter = chunk_iter(list, pre_masks);
    let mut last_off = 0;
    for (off, (chunk, mask)) in iter {
        for _ in (last_off..off).step_by(64) {masks.push(0)};
        masks.push(frustum_planes_triangle_chunk_mask(&fr, &chunk, mask));
        last_off = off + 64;
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
    let ts = tile_size as f32;
    let tw = (dim[0] as f32 / ts).ceil() * ts;
    let th = (dim[1] as f32 / ts).ceil() * ts;
    let mix = i as f32 * ts;
    let miy = j as f32 * ts;
    let max = (i + 1) as f32 * ts;
    let may = (j + 1) as f32 * ts;
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

/// Prepare row sub-tile compressed masks.
pub fn pre_row_sub_masks(dim: PixelPos, n_tile_size: u32) -> Vec<Vec<CompressedMasks>> {
    let h = tile_grid(dim, n_tile_size)[1];
    vec![vec![]; h as usize]
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

/// Collect sub-masks per tile row.
pub fn row_sub_masks<T: Produce<Triangle> + ?Sized + Sync>(
    persp: &CameraPerspective,
    dim: PixelPos,
    n_tile_size: u32,
    grid: [u32; 2],
    triangle_limit_per_tile: u32,
    list: &T,
    masks: &[CompressedMasks],
    sub_masks: &mut Vec<Vec<CompressedMasks>>
) {
    use rayon::prelude::*;

    let ndim = near_dim(&persp);
    let w = grid[0];
    sub_masks.par_iter_mut().enumerate().for_each(|(tj, sm)| {
        let tj = tj as u32;
        sm.clear();
        for (ti, val) in row_sub_tile_iter(n_tile_size, grid, tj, triangle_limit_per_tile, masks) {
            if let Some((st, offset)) = val {
                let k = tj * w + ti;
                let pre_masks = &masks[k as usize];
                let n = n_tile_size / st;
                assert_eq!(sm.len(), offset);
                for j in 0..n {
                    for i in 0..n {
                        let ind: usize = offset + (j * n + i) as usize;
                        while sm.len() <= ind {
                            sm.push(CompressedMasks::new())
                        }
                        let m = &mut sm[ind];
                        m.clear();
                        let pos = [ti * n + i, tj * n + j];
                        let tpos = tile_pos(dim, pos, st);
                        let tsize = tile_size(dim, pos, st);
                        tile_mask_with_pre_mask(persp, ndim, tpos, tsize, list, m, pre_masks);
                    }
                }
            }
        }
    });
}

/// Collect all masks per tile, using pre-masks at lower resolution.
///
/// This speeds up compression, because one can iterate faster over
/// the virtual triangles using the pre-masks.
///
/// `scale_to_pre_tile_size` specifies the ratio of pre-tile-size divided by tile-size.
pub fn masks_with_pre_masks<T: Produce<Triangle> + ?Sized + Sync>(
    persp: &CameraPerspective,
    dim: PixelPos,
    n_tile_size: u32,
    scale_to_pre_tile_size: u32,
    list: &T,
    masks: &mut [CompressedMasks],
    pre_masks: &mut [CompressedMasks],
) {
    use rayon::prelude::*;

    let s = scale_to_pre_tile_size;
    let w = tile_grid(dim, n_tile_size)[0];
    let w2 = tile_grid(dim, n_tile_size * s)[0];
    let ndim = near_dim(&persp);
    masks.par_iter_mut().enumerate().for_each(|(k,  masks)| {
        masks.clear();
        let i = k as u32 % w;
        let j = k as u32 / w;
        let pre_masks = &pre_masks[((j / s) * w2 + i / s) as usize];
        let tpos = tile_pos(dim, [i, j], n_tile_size);
        let tsize = tile_size(dim, [i, j], n_tile_size);
        tile_mask_with_pre_mask(persp, ndim, tpos, tsize, list, masks, pre_masks);
    });
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
pub fn render_tile_depth<T: Produce<Triangle> + ?Sized, const TILE_SIZE: usize>(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    list: &T,
    masks: &CompressedMasks,
    tile: &mut [[RayHit; TILE_SIZE]; TILE_SIZE],
) {
    let n_tile_size = TILE_SIZE as u32;
    let eye = [0.0; 3];
    let iter = chunk_iter(list, masks);
    for (off, (chunk, mask)) in iter {
        for j in 0..n_tile_size {
            for i in 0..n_tile_size {
                let dir: Point = ray_dir(persp, eye, [pos[0] + i, pos[1] + j], dim);
                ray_triangle_chunk_hit_update((eye, dir), &chunk, mask, off,
                    &mut tile[j as usize][i as usize]);
            }
        }
    }
}

/// Creates an iterator for row sub-tiling, where `(col, None)` means no sub-tiling and
/// `(col, Some((st, offset)))` means sub-tiling of size `st` with sub-mask offset.
///
/// The sub-mask offset is relative to some list per row that
/// stores compressed masks for sub-tiling.
pub fn row_sub_tile_iter(
    n_tile_size: u32,
    grid: [u32; 2],
    row: u32,
    triangle_limit_per_tile: u32,
    masks: &[CompressedMasks]
) -> impl Iterator<Item = (u32, Option<(u32, usize)>)> {
    let w = grid[0];
    (0..w).scan(0usize, move |offset, col| {
        let k = row * w + col;
        let triangles = masks[k as usize].count_ones() as u32;
        Some((col, if triangles >= triangle_limit_per_tile {
            let st = sub_tile(n_tile_size, triangles, triangle_limit_per_tile);
            let n = n_tile_size / st;
            let start = *offset;
            *offset += (n * n) as usize;
            Some((st, start))
        } else {None}))
    })
}

/// Render depth of row of sub-tiles using a camera perspective, image resolution,
/// tile position, tile size, sub-tile size and triangle list with mask,
/// into a tile depth and index buffer.
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
pub fn render_row_sub_tile_depth_all<T: Produce<Triangle> + ?Sized, const TILE_SIZE: usize>(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    sub_tile_size: u32,
    list: &T,
    sub_masks: &[CompressedMasks],
    tile: &mut [[RayHitAll; TILE_SIZE]; TILE_SIZE],
) -> bool {
    use crate::IndexFlag;

    let n_tile_size = TILE_SIZE as u32;
    let n = n_tile_size / sub_tile_size;
    let eye = [0.0; 3];
    let mut alive = false;

    // For each sub-tile.
    for kj in 0..n {
        for ki in 0..n {
            // Iterate through the chunks for that specific sub-tile.
            let k = kj * n + ki;
            let iter = chunk_iter(list, &sub_masks[k as usize]);
            let sub_tile_pos = [ki * sub_tile_size, kj * sub_tile_size];
            for (off, (chunk, mask)) in iter {
                // For each ray in the sub-tile.
                for j in 0..sub_tile_size {
                    for i in 0..sub_tile_size {
                        let i = sub_tile_pos[0] + i;
                        let j = sub_tile_pos[1] + j;

                        let hit = &mut tile[j as usize][i as usize];
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
            }
        }
    }

    // Terminate rays when not hitting anything new.
    let len = list.virtual_length();
    for j in 0..TILE_SIZE {
        for i in 0..TILE_SIZE {
            let hit = &mut tile[j][i];
            if let Some((_, index_flag)) = hit {
                if !index_flag.flag() || index_flag.index() >= len {*hit = None};
            }
        }
    }

    alive
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
pub fn render_tile_depth_all<T: Produce<Triangle> + ?Sized, const TILE_SIZE: usize>(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    list: &T,
    masks: &CompressedMasks,
    tile: &mut [[RayHitAll; TILE_SIZE]; TILE_SIZE],
) -> bool {
    use crate::IndexFlag;

    let n_tile_size = TILE_SIZE as u32;
    let eye = [0.0; 3];
    let iter = chunk_iter(list, masks);
    let mut alive = false;
    for (off, (chunk, mask)) in iter {
        let mut inner_alive = false;
        for j in 0..n_tile_size {
            for i in 0..n_tile_size {
                let hit = &mut tile[j as usize][i as usize];
                let dir: Point = ray_dir(persp, eye, [pos[0] + i, pos[1] + j], dim);
                ray_triangle_chunk_hit_all_update((eye, dir), &chunk, mask, off, hit);
                if let Some((d, index_flag)) = hit {
                    if !index_flag.flag() {
                        let ind = index_flag.index();
                        let new_ind = ind.max(off + 64);
                        *hit = Some((*d, IndexFlag::from_parts(new_ind, false)));
                    }
                }
                inner_alive |= hit.is_some();
            }
        }
        alive |= inner_alive;
        // Skip iteration if there no rays alive or nothing to render.
        if !inner_alive {return alive}
    }

    // Terminate rays when not hitting anything new.
    let len = list.virtual_length();
    for j in 0..TILE_SIZE {
        for i in 0..TILE_SIZE {
            let hit = &mut tile[j][i];
            if let Some((_, index_flag)) = hit {
                if !index_flag.flag() || index_flag.index() >= len {*hit = None};
            }
        }
    }

    alive
}
