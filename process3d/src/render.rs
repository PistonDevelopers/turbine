//! Rendering.

use crate::color::*;
use crate::ray::*;
use crate::tile::*;
use crate::profile::*;
use crate::produce::*;
use crate::math::*;
use crate::acc::*;
use crate::frustrum::depth_linear;
use crate::mask::CompressedMasks;
use crate::cam::{Camera, CameraPerspective};
use crate::{
    IndexFlag,
    PixelPos,
    RayHit,
    Rgba,
    Triangle,
    Vector,
};

/// Stores arguments for shaders.
pub struct ShaderData<Args> {
    /// The ray depth and index of graphics primitive.
    pub hit: RayHit,
    /// The depth in range `0.0` to `1.0` where `0.0` is near clip plane
    /// and `1.0` is far clip plane.
    pub depth_linear: f32,
    /// The internal address of hit graphics primitive in producer.
    ///
    /// For example, when a triangle is hit,
    /// the triangle might belong to some voxel in internal address space.
    /// In this case, the produces generates triangles from voxels.
    /// With ther words, this value tells which voxel gets hit.
    pub internal_offset: Option<usize>,
    /// Customized arguments to the shader.
    pub args: Args,
}

/// The type of shader.
///
/// A shader might modify the default color before accumulation.
pub type Shader<Color, Args> = fn(&mut Color, ShaderData<Args>);

/// Stores data needed during rendering.
pub struct Renderer<'a, Scene, Prod, Img, A, ShaderArgs, P>
    where Scene: Sync, Prod: Produce<Triangle> + Sync + ?Sized, A: Acc,
{
    /// Scene data.
    pub scene: Scene,
    /// Gets a default ray color prior to shading.
    ///
    /// Make sure to convert to the same color space as the accumulator.
    ///
    /// For example, if the accumulator uses linear color space,
    /// and you use sRGB color space in scene data,
    /// then you should convert to linear color space.
    pub scene_ray_color: fn(&Scene, f32, usize) -> (A::In, ShaderArgs),
    /// A customized shader transform prior to color accumulation.
    ///
    /// This can be used to change the color of the ray.
    pub shader: Shader<A::In, ShaderArgs>,
    /// Returns `true` if color is transparent, `false` otherwise.
    ///
    /// This is used to filter out colors that do not contribute in accumulator.
    pub is_transparent: fn(&A::In) -> bool,
    /// Produces the final color from accumulator.
    pub acc_to_linear_rgba: fn(A::Out) -> Rgba,
    /// A producer.
    pub producer: &'a Prod,
    /// The target image.
    pub img: &'a mut Img,
    /// Get the size of the image in pixels.
    pub size: fn(&Img) -> PixelPos,
    /// Writes pixel to image.
    pub pxl: fn(&mut Img, PixelPos, c: Rgba<u8>),
    /// Accumulator data.
    ///
    /// This is used to pre-configure the accumulator with some data.
    pub acc_data: A::Data,
    /// The camera perspective.
    pub persp: &'a CameraPerspective,
    /// The camera.
    pub cam: &'a Camera,
    /// Can be used to scale or flip axis.
    pub flip_xyz: Vector,
    /// Stores compressed masks per render tile.
    pub compr_masks: &'a mut [CompressedMasks],
    /// Stores compressed masks for pre-pre-processing.
    pub pre_compr_masks: &'a mut [CompressedMasks],
    /// Stores compressed masks for adaptive sub-tiling.
    pub sub_compr_masks: &'a mut Vec<Vec<CompressedMasks>>,
    /// A limit on the number of triangles per tile before doing adaptive sub-tiling.
    ///
    /// Adaptive sub-tiling produces sub-tiles such that the average number of triangles
    /// is less than this limit, down to 2x2 sub-tiles.
    pub sub_tile_triangle_limit: u32,
    /// Data used to store performance profiling information.
    pub profile: &'a mut P,
    /// Reports the amount of seconds taken to render (`None` if profiling is disabled).
    pub profile_render: fn(&mut P, Option<f64>),
    /// Reports profile compress data and amount of seconds (`None` if profiling is disabled).
    pub profile_compress: fn(&mut P, ProfileCompressData, Option<f64>),
    /// Whether to use adaptive sub-tiling.
    ///
    /// Adaptive sub-tiling splits tiles that have a large amount
    /// of triangles into sub-tiles, where the average amount of triangles
    /// is less thatn `sub_tile_triangle_limit`.
    pub sub_masks: bool,
    /// Whether to use pre-masks.
    ///
    /// This is a pre-pre-processing step where run-length compression
    /// on masks uses a lower resolution, to speed up pre-processing.
    pub pre_masks: bool,
    /// Whether profiling is enabled.
    ///
    /// Profiling gathers some data about the rendering process.
    pub profile_enabled: bool,
    /// A limit of how many accumulations per tile.
    ///
    /// Is used to prevent inner infinite rendering loops.
    ///
    /// This is usually some multiple of the accumulator buffer size.
    pub acc_limit: u32,
    /// The scale ratio between pre-tile size and tile size.
    pub scale_to_pre_tile_size: u32,
}

impl<Scene, Prod, Img, Accumulator, ShaderArgs, P>
Renderer<'_, Scene, Prod, Img, Accumulator, ShaderArgs, P>
    where Scene: Sync,
          Prod: Produce<Triangle> + Sync + ?Sized,
          Accumulator: Acc,
{
    /// Render with some render tile size.
    ///
    /// The tile size should be optimized for adaptive sub-tile rendering.
    /// Use `optimal_sub_tile_size`.
    pub fn render<const TILE_SIZE: usize>(self) {
        let Renderer {
            scene, scene_ray_color, producer,
            img, size, pxl, acc_data, persp, cam, flip_xyz,
            compr_masks, pre_compr_masks, sub_compr_masks,
            sub_tile_triangle_limit, shader, profile, profile_render,
            sub_masks, pre_masks, profile_enabled, profile_compress,
            acc_limit, scale_to_pre_tile_size, is_transparent, acc_to_linear_rgba,
        } = self;

        let profile_without_sub_masks = !sub_masks;
        let profile_without_pre_masks = !pre_masks;

        use rayon::prelude::*;
        use std::sync::mpsc::channel;
        use vecmath::row_mat4_mul;

        let [sx, sy, sz] = flip_xyz;
        let flip = [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, sz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let view = mat4_transposed(cam.orthogonal());
        let view = row_mat4_mul(flip, view);

        let producer: &TransformProducer<_> = &TransformProducer {
            matrix: view,
            inner: producer,
        };

        let size = (size)(img);
        let [w, h] = size;

        let tile_size = TILE_SIZE as u32;
        let grid = tile_grid(size, tile_size);

        let start: Option<f64> = if profile_enabled {Some(now())} else {None};

        if profile_without_pre_masks {
            masks(
                persp,
                size,
                tile_size,
                producer,
                compr_masks,
            );
        } else {
            masks(
                persp,
                size,
                tile_size * scale_to_pre_tile_size,
                producer,
                pre_compr_masks,
            );
            masks_with_pre_masks(
                persp,
                size,
                tile_size,
                scale_to_pre_tile_size,
                producer,
                compr_masks,
                pre_compr_masks,
            );
        }

        let koeff: u32 = sub_tile_triangle_limit;
        if !profile_without_sub_masks {
            row_sub_masks(&persp, size, tile_size, grid, koeff, producer,
                compr_masks, sub_compr_masks);
        }

        profile_compress(profile, ProfileCompressData {tile_size, grid, compr_masks}, start);

        let (tx, rx) = channel();

        let start: Option<f64> = if profile_enabled {Some(now())} else {None};
        (0..grid[1]).into_par_iter().for_each_with(tx, |tx, tj| {
            let nh = (tj + 1) * tile_size;
            let th = nh.min(h) - tj * tile_size;
            let mut depth_buffer = &mut [[None; TILE_SIZE]; TILE_SIZE];
            let mut acc = Accumulator::new(acc_data.clone());
            let sm = &sub_compr_masks[tj as usize];
            for (ti, val) in row_sub_tile_iter(tile_size, grid, tj, koeff, compr_masks) {
                let masks = &compr_masks[(tj * grid[0] + ti) as usize];
                let triangles = masks.count_ones() as u32;
                if triangles == 0 {continue};

                acc.clear();

                let nw = (ti + 1) * tile_size;
                let tw = nw.min(w) - ti * tile_size;
                let pos = [ti * tile_size, tj * tile_size];

                // Stores depth and colors that the ray accumulates.
                let mut write = [[[0; 4]; TILE_SIZE]; TILE_SIZE];
                *depth_buffer = [[
                    Some((0.0, IndexFlag::from_parts(0, false))); TILE_SIZE]; TILE_SIZE];

                for _ in 0..acc_limit {
                    match (profile_without_sub_masks, val) {
                        (true, _) | (false, None) => {
                            if !render_tile_depth_all(&persp, size, pos,
                                producer, masks, &mut depth_buffer) {break};
                        }
                        (false, Some((st, offset))) => {
                            if !render_row_sub_tile_depth_all(&persp, size, pos, st,
                                producer, &sm[offset..], &mut depth_buffer) {break};
                        }
                    }

                    for j in 0..th {
                        for i in 0..tw {
                            let hit = &mut depth_buffer[j as usize][i as usize];
                            *hit = if let Some((depth, ind)) = *hit {
                                if !ind.flag() {continue};

                                let internal_offset = producer.to_internal(ind.index());
                                let (mut color, args) = scene_ray_color(
                                    &scene, depth, internal_offset.unwrap());

                                let hit = ray_hit_all_to_ray_hit(*hit);
                                shader(&mut color, ShaderData {
                                    hit: hit,
                                    depth_linear: depth_linear(&persp, hit),
                                    internal_offset,
                                    args,
                                });

                                if !is_transparent(&color) {acc.upd(i, j, depth, color)};
                                Some((depth, IndexFlag::from_parts(ind.index() + 1, false)))
                            } else {None}
                        }
                    }
                }

                for j in 0..th {
                    for i in 0..tw {
                        write[j as usize][i as usize] =
                            rgba_to_u8(rgba_gamma_linear_to_srgb(acc_to_linear_rgba(acc.acc(i, j))));
                    }
                }

                let _ = tx.send(([ti * tile_size, tj * tile_size], write));
            }
        });

        for y in 0..h {
            for x in 0..w {
                pxl(img, [x, y], [0; 4]);
            }
        }

        for (offset, tile) in rx {
            for j in 0..TILE_SIZE as u32 {
                for i in 0..TILE_SIZE as u32 {
                    let color = tile[j as usize][i as usize];
                    let x = offset[0] + i;
                    let y = offset[1] + j;
                    if x >= w || y >= h {continue};
                    let y = h - y - 1;
                    pxl(img, [x, y], color);
                }
            }
        }

        profile_render(profile, start);
    }
}
