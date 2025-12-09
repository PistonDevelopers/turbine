//! # Helper functions for colors

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

/// Converts color to `f32` precision.
pub fn rgba_to_f32(a: Rgba<u8>) -> Rgba {
    [
        (a[0] as f32) / 255.0,
        (a[1] as f32) / 255.0,
        (a[2] as f32) / 255.0,
        (a[3] as f32) / 255.0,
    ]
}

/// Converts color to `u8` precision.
pub fn rgba_to_u8(a: Rgba) -> Rgba<u8> {
    use crate::math::clamp;
    [
        (clamp(a[0]) * 255.0) as u8,
        (clamp(a[1]) * 255.0) as u8,
        (clamp(a[2]) * 255.0) as u8,
        (clamp(a[3]) * 255.0) as u8,
    ]
}

/// Alpha blending over operation in linear color space.
pub fn rgba_alpha_blending_linear_over(a: Rgba, b: Rgba) -> Rgba {
    let alpha = a[3] + b[3] * (1.0 - a[3]);
    [
        a[0] * a[3] + b[0] * b[3] * (1.0 - a[3]),
        a[1] * a[3] + b[1] * b[3] * (1.0 - a[3]),
        a[2] * a[3] + b[2] * b[3] * (1.0 - a[3]),
        alpha,
    ]
}

/// Alpha blending over operation in sRGB color space.
pub fn rgba_alpha_blending_srgb_over(a: Rgba, b: Rgba) -> Rgba {
    let a = rgba_gamma_srgb_to_linear(a);
    let b = rgba_gamma_srgb_to_linear(b);
    rgba_gamma_linear_to_srgb(rgba_alpha_blending_linear_over(a, b))
}

/// Alpha blending over operation in sRGB color space with `u8` precision.
pub fn rgba_alpha_blending_srgb_over_u8(a: Rgba<u8>, b: Rgba<u8>) -> Rgba<u8> {
    rgba_to_u8(rgba_alpha_blending_srgb_over(rgba_to_f32(a), rgba_to_f32(b)))
}
