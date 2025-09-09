//! Tile rendering algorithms.

use crate::{PixelPos, Point, TilePos, Triangle, Uv};
use crate::cam::CameraPerspective;
use crate::frustrum::{
    frustum_planes_tile,
    frustum_planes_triangle_chunk_mask,
    near_dim,
};
use crate::mask::CompressedMasks;
use crate::ray::{ray_dir, ray_triangle_chunk_hit_update};
use crate::triangle::{chunk_iter, triangle_chunk};

/// From camera perspective, tile and a list of triangles, get mask of intersecting triangles.
///
/// This is used as a preparation stage before sampling each tile in parallel.
///
/// The algorithm does not clear the masks before pushing new ones.
pub fn tile_mask(
    persp: &CameraPerspective,
    dim: Uv,
    tile_pos: Uv,
    tile_size: Uv,
    list: &[Triangle],
    masks: &mut CompressedMasks,
) {
    let fr = frustum_planes_tile(persp, dim, tile_pos, tile_size);
    let mut i = 0;
    let len = masks.len();
    loop {
        let (chunk, bits) = triangle_chunk(&list[i..]);
        masks.push(frustum_planes_triangle_chunk_mask(&fr, &chunk, bits));
        if masks.len() >= (len + list.len() / 64) {break}
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

/// Collect all masks per tile.
pub fn masks(
    persp: &CameraPerspective,
    dim: PixelPos,
    n_tile_size: u32,
    list: &[Triangle],
) -> Vec<CompressedMasks> {
    let [w, h] = tile_grid(dim, n_tile_size);
    let n = w * h;
    let mut ret: Vec<CompressedMasks> = Vec::with_capacity(n as usize);
    let ndim = near_dim(&persp);
    for j in 0..h {
        for i in 0..w {
            let mut masks = CompressedMasks::new();
            let tpos = tile_pos(dim, [i, j], n_tile_size);
            let tsize = tile_size(dim, [i, j], n_tile_size);
            tile_mask(persp, ndim, tpos, tsize, list, &mut masks);
            ret.push(masks);
        }
    }
    ret
}

/// Render depth of a tile using a camera perspective, image resolution,
/// tile position, tile size and triangle list with mask, into a tile depth and index buffer.
///
/// Iterates through triangle chunks and updates the tile depth and index buffer.
///
/// Ray direction is recreated for each triangle chunk.
///
/// Requires compressed masks per tile to be prepared in advance.
pub fn render_tile_depth(
    persp: &CameraPerspective,
    dim: PixelPos,
    pos: PixelPos,
    n_tile_size: u32,
    list: &[Triangle],
    masks: &CompressedMasks,
    tile: &mut [Option<(f32, usize)>],
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
