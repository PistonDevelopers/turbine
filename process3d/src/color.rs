//! Helper functions for colors.

use crate::{Rgb, Rgba};

#[inline(always)]
fn component_srgb_to_linear(f: f32) -> f32 {
    if f <= 0.04045 {
        f / 12.92
    } else {
        ((f + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts gamma (brightness) from sRGB to linear color space.
///
/// sRGB is the default color space for image editors, pictures, internet etc.
/// Linear gamma yields better results when doing math with colors.
pub fn rgb_gamma_srgb_to_linear(c: Rgb) -> Rgb {
    [
        component_srgb_to_linear(c[0]),
        component_srgb_to_linear(c[1]),
        component_srgb_to_linear(c[2]),
    ]
}

/// Converts gamma (brightness) from sRGB to linear color space.
///
/// sRGB is the default color space for image editors, pictures, internet etc.
/// Linear gamma yields better results when doing math with colors.
pub fn rgba_gamma_srgb_to_linear(c: Rgba) -> Rgba {
    [
        component_srgb_to_linear(c[0]),
        component_srgb_to_linear(c[1]),
        component_srgb_to_linear(c[2]),
        c[3],
    ]
}

#[inline(always)]
fn component_linear_to_srgb(f: f32) -> f32 {
    if f <= 0.0031308 {
        f * 12.92
    } else {
        1.055 * f.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts gamma (brightness) of a color from linear color space to sRGB.
///
/// sRGB is the default color space for image editors, pictures, internet etc.
/// Linear gamma yields better results when doing math with colors.
pub fn rgb_gamma_linear_to_srgb(c: Rgb) -> Rgb {
    [
        component_linear_to_srgb(c[0]),
        component_linear_to_srgb(c[1]),
        component_linear_to_srgb(c[2]),
    ]
}

/// Converts gamma (brightness) of a color from linear color space to sRGB.
///
/// sRGB is the default color space for image editors, pictures, internet etc.
/// Linear gamma yields better results when doing math with colors.
pub fn rgba_gamma_linear_to_srgb(c: Rgba) -> Rgba {
    [
        component_linear_to_srgb(c[0]),
        component_linear_to_srgb(c[1]),
        component_linear_to_srgb(c[2]),
        c[3],
    ]
}
