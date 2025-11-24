//! # Fog
//!
//! This module contains functionality to process fog effects.

use crate::Rgba;

/// Calculates effect of volumetric fog for uniform density `p` and distance.
///
/// Returns `1.0` when maximum density and `0.0` for zero density.
pub fn volumetric_fog_fx(p: f32, dist: f32) -> f32 {1.0 - (-p * dist).exp()}

/// Calculates alpha channel using effect of volumetric fog.
///
/// This version of volumetric fog retreats to opaque colors for high alpha values.
pub fn volumetric_fog_alpha(alpha: f32, fx: f32, dist: f32) -> f32 {
    let new_alpha = alpha * fx;
    let t = alpha * if dist < std::f32::EPSILON {0.0} else {1.0};
    alpha * t + new_alpha * (1.0 - t)
}

/// Represents the fog state for accumulators using volumetric fog effects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FogState {
    /// No previous fog effect in progress.
    None,
    /// Open start of fog volume ray interval.
    Start {
        /// The color of the fog.
        color: Rgba,
        /// The start depth of fog volume ray interval.
        depth: f32,
        /// Total distance of previous fog volume ray intervals.
        distance: f32
    },
    /// Closed end of fog volume ray interval.
    End {
        /// The color of the fog.
        color: Rgba,
        /// Distance used to render fog effect.
        distance: f32
    },
    /// Commit fog effect contribution to final color accumulation.
    Commit {
        /// The color of the fog.
        color: Rgba,
        /// Distance to render fog effect.
        distance: f32
    },
}

impl FogState {
    /// Updates the fog state using depth `d` and color `c`.
    pub fn update(self, d: f32, c: Rgba) -> FogState {
        use FogState::*;
        match self {
            None | Commit {..} =>
                if c[3] < 1.0 {Start {color: c, depth: d, distance: 0.0}} else {None},
            End {color, distance} => {
                if c == color {Start {color, depth: d, distance}}
                else {Commit {color, distance}}
            }
            Start {color, depth, distance} => {
                let distance = (d - depth).abs() + distance;
                if c == color {End {color, distance}}
                else {Commit {color, distance}}
            }
        }
    }

    /// Accumulation using alpha blend over in sRGB color space.
    pub fn acc_alpha_blend_srgb_over(&mut self, d: f32, c: Rgba, acc_color: &mut Rgba) {
        use crate::color::rgba_alpha_blending_srgb_over;

        // Update state before deciding what to do next.
        *self = self.update(d, c);
        match self {
            FogState::None => {
                *acc_color = rgba_alpha_blending_srgb_over(*acc_color, c);
            }
            FogState::Start {..} | FogState::End {..} => {}
            FogState::Commit {color: fog_color, distance} => {
                let fx = volumetric_fog_fx(fog_color[3], *distance);
                let [r, g, b, a] = *fog_color;
                let c2 = [r, g, b, volumetric_fog_alpha(a, fx, *distance)];
                *acc_color = rgba_alpha_blending_srgb_over(*acc_color, c2);
                if c[3] < 1.0 {
                    *self = FogState::Start {color: c, depth: d, distance: 0.0};
                } else {
                    *self = FogState::None;
                    *acc_color = rgba_alpha_blending_srgb_over(*acc_color, c);
                }
            }
        }
    }

    /// End of accumulation using alpha blend over in sRGB color space.
    pub fn acc_end_alpha_blend_srgb_over(&mut self, acc_color: &mut Rgba) {
        use crate::color::rgba_alpha_blending_srgb_over;

        match self {
            FogState::None => {}
            FogState::Start {..} => {}
            FogState::Commit {..} => {}
            FogState::End {color: fog_color, distance} => {
                let fx = volumetric_fog_fx(fog_color[3], *distance);
                let [r, g, b, a] = *fog_color;
                let c = [r, g, b, volumetric_fog_alpha(a, fx, *distance)];
                *acc_color = rgba_alpha_blending_srgb_over(*acc_color, c);
            }
        }
    }

    /// Accumulation using alpha blend over in lineaer color space.
    pub fn acc_alpha_blend_linear_over(&mut self, d: f32, c: Rgba, acc_color: &mut Rgba) {
        use crate::color::rgba_alpha_blending_linear_over;

        *self = self.update(d, c);
        match self {
            FogState::None => {
                *acc_color = rgba_alpha_blending_linear_over(*acc_color, c);
            }
            FogState::Start {..} |
            FogState::End {..} => {}
            FogState::Commit {color: fog_color, distance} => {
                let fx = volumetric_fog_fx(fog_color[3], *distance);
                let [r, g, b, a] = *fog_color;
                let c2 = [r, g, b, volumetric_fog_alpha(a, fx, *distance)];
                *acc_color = rgba_alpha_blending_linear_over(*acc_color, c2);
                if c[3] < 1.0 {
                    *self = FogState::Start {color: c, depth: d, distance: 0.0};
                } else {
                    *self = FogState::None;
                    *acc_color = rgba_alpha_blending_linear_over(*acc_color, c);
                }
            }
        }
    }

    /// End of accumulation using alpha blend over in linear color space.
    pub fn acc_end_alpha_blend_linear_over(&mut self, acc_color: &mut Rgba) {
        use crate::color::rgba_alpha_blending_linear_over;

        match self {
            FogState::None |
            FogState::Start {..} => {}
            FogState::Commit {color, distance} |
            FogState::End {color, distance} => {
                let fx = volumetric_fog_fx(color[3], *distance);
                let [r, g, b, a] = *color;
                let c = [r, g, b, volumetric_fog_alpha(a, fx, *distance)];
                *acc_color = rgba_alpha_blending_linear_over(*acc_color, c);
            }
        }
    }
}
