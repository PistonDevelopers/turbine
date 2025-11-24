//! # Accumulators
//!
//! Accumulators provide an abstraction for rendering pipelines
//! that allows applying post-processing effects.
//!
//! An accumulator might be used in a rendering pipeline
//! for semi-transparent objects where every triangle per tile is visited.
//!
//! There are some example Accumulators in this module.

use crate::{
    fog::FogState,
    Rgba,
};

/// Implemented by accumulators.
///
/// An accumulator is usually operating in tile coordinates.
///
/// When tiles are split into rows per thread,
/// you can reuse the accumulator for each row.
pub trait Acc {
    /// Constructor data type.
    type Data: Clone + Sync;
    /// Input type.
    type In;
    /// Output type.
    type Out;
    /// Initialize the accumulator.
    fn new(data: Self::Data) -> Self;
    /// Reset the accumulator.
    fn clear(&mut self);
    /// Update the accumulator with new data.
    fn upd(&mut self, i: u32, j: u32, depth: f32, data: Self::In);
    /// Calculate the final output.
    fn acc(&self, i: u32, j: u32) -> Self::Out;
}

/// Accumulator for tile rendering, using Rgba colors in sRGB color space
/// and semi-fog effect for alpha over blending.
///
/// This is used to compose colors from triangles into a final color.
pub struct TileRgbaSrgbSemiFogAcc<const TILE_SIZE: usize, const ACC: usize> {
    /// Stores buffer data of the accumulator.
    pub buf: [[[(f32, Rgba); ACC]; TILE_SIZE]; TILE_SIZE],
    /// Stores the buffer lengths.
    pub len: [[u8; TILE_SIZE]; TILE_SIZE],
}

impl<const TILE_SIZE: usize, const ACC: usize> Acc for TileRgbaSrgbSemiFogAcc<TILE_SIZE, ACC> {
    type Data = ();
    type In = Rgba;
    type Out = Rgba;
    fn new(_: ()) -> Self {
        Self {
            buf: [[[(0.0, [0.0; 4]); ACC]; TILE_SIZE]; TILE_SIZE],
            len: [[0; TILE_SIZE]; TILE_SIZE],
        }
    }
    fn clear(&mut self) {
        self.buf = [[[(0.0, [0.0; 4]); ACC]; TILE_SIZE]; TILE_SIZE];
        self.len = [[0; TILE_SIZE]; TILE_SIZE];
    }
    fn upd(&mut self, i: u32, j: u32, depth: f32, color: Rgba) {
        let len = {
            let len = &mut self.len[j as usize][i as usize];
            if *len as usize >= ACC {return};

            *len += 1;
            *len
        };
        self.ins(len, i, j, depth, color);
    }
    fn acc(&self, i: u32, j: u32) -> Rgba {
        let len = self.len[j as usize][i as usize];
        let buf = &self.buf[j as usize][i as usize];
        let mut color = [0.0; 4];
        let mut fog = FogState::None;
        for k in 0..len {
            let (d, c) = buf[k as usize];
            fog.acc_alpha_blend_srgb_over(d, c, &mut color);
        }
        fog.acc_end_alpha_blend_srgb_over(&mut color);
        color
    }
}

impl<const TILE_SIZE: usize, const ACC: usize> TileRgbaSrgbSemiFogAcc<TILE_SIZE, ACC> {
    fn ins(&mut self, len: u8, i: u32, j: u32, mut depth: f32, mut color: Rgba) {
        let buf = &mut self.buf[j as usize][i as usize];
        for k in 0..len {
            let (d, c) = buf[k as usize];
            if d == 0.0 {
                buf[k as usize] = (depth, color);
                return;
            } else if d > depth {
                buf[k as usize] = (depth, color);

                // There is no need to look behind opaque colors.
                if color[3] >= 1.0 {
                    self.len[j as usize][i as usize] = k + 1;
                    return
                };

                depth = d;
                color = c;
            }
        }
    }
}

/// Accumulator for tile rendering, using Rgba colors in linear color space
/// and semi-fog effect for alpha over blending.
///
/// This is used to compose colors from triangles into a final color.
pub struct TileRgbaLinearSemiFogAcc<const TILE_SIZE: usize, const ACC: usize> {
    /// Stores buffer data of the accumulator.
    pub buf: [[[(f32, Rgba); ACC]; TILE_SIZE]; TILE_SIZE],
    /// Stores the buffer lengths.
    pub len: [[u8; TILE_SIZE]; TILE_SIZE],
}

impl<const TILE_SIZE: usize, const ACC: usize> Acc for TileRgbaLinearSemiFogAcc<TILE_SIZE, ACC> {
    type Data = ();
    type In = Rgba;
    type Out = Rgba;
    fn new(_: ()) -> Self {
        Self {
            buf: [[[(0.0, [0.0; 4]); ACC]; TILE_SIZE]; TILE_SIZE],
            len: [[0; TILE_SIZE]; TILE_SIZE],
        }
    }
    fn clear(&mut self) {
        self.buf = [[[(0.0, [0.0; 4]); ACC]; TILE_SIZE]; TILE_SIZE];
        self.len = [[0; TILE_SIZE]; TILE_SIZE];
    }
    fn upd(&mut self, i: u32, j: u32, depth: f32, color: Rgba) {
        let len = {
            let len = &mut self.len[j as usize][i as usize];
            if *len as usize >= ACC {return};

            *len += 1;
            *len
        };
        self.ins(len, i, j, depth, color);
    }
    fn acc(&self, i: u32, j: u32) -> Rgba {
        let len = self.len[j as usize][i as usize];
        let buf = &self.buf[j as usize][i as usize];
        let mut color = [0.0; 4];
        let mut fog = FogState::None;
        for k in 0..len {
            let (d, c) = buf[k as usize];
            fog.acc_alpha_blend_linear_over(d, c, &mut color);
        }
        fog.acc_end_alpha_blend_linear_over(&mut color);
        color
    }
}

impl<const TILE_SIZE: usize, const ACC: usize> TileRgbaLinearSemiFogAcc<TILE_SIZE, ACC> {
    fn ins(&mut self, len: u8, i: u32, j: u32, mut depth: f32, mut color: Rgba) {
        let buf = &mut self.buf[j as usize][i as usize];
        for k in 0..len {
            let (d, c) = buf[k as usize];
            if d == 0.0 {
                buf[k as usize] = (depth, color);
                return;
            } else if d > depth {
                buf[k as usize] = (depth, color);

                // There is no need to look behind opaque colors.
                if color[3] >= 1.0 {
                    self.len[j as usize][i as usize] = k + 1;
                    return
                };

                depth = d;
                color = c;
            }
        }
    }
}
