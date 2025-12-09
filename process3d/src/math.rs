//! # Mathematical algorithms

pub use vecmath::{
    mat4_id,
    mat4_transposed,
};

use crate::{Chunk, Cube, Line, Matrix4, Point, Quad, Rgb, Rgba, Triangle};

/// Transform point.
///
/// This transform is row-major, which is standard mathematical notation.
/// To convert column-major to row-major, use `mat4_transposed`.
///
/// Notice! In OpenGL matrices are column-major.
pub fn transform_point(mat: &Matrix4, p: Point) -> Point {
    [
        mat[0][0] * p[0] + mat[0][1] * p[1] + mat[0][2] * p[2] + mat[0][3],
        mat[1][0] * p[0] + mat[1][1] * p[1] + mat[1][2] * p[2] + mat[1][3],
        mat[2][0] * p[0] + mat[2][1] * p[1] + mat[2][2] * p[2] + mat[2][3],
    ]
}

/// Transform triangle.
///
/// This transform is row-major, which is standard mathematical notation.
/// To convert column-major to row-major, use `mat4_transposed`.
///
/// Notice! In OpenGL matrices are column-major.
pub fn transform_triangle(mat: &Matrix4, (a, b, c): Triangle) -> Triangle {
    (transform_point(mat, a), transform_point(mat, b), transform_point(mat, c))
}

/// Transforms chunk.
pub fn transform_chunk(mat: &Matrix4, chunk: &mut Chunk<Triangle>) {
    for i in 0..64 {
        chunk[i] = transform_triangle(mat, chunk[i]);
    }
}

/// Transform quad.
///
/// This transform is row-major, which is standard mathematical notation.
/// To convert column-major to row-major, use `mat4_transposed`.
///
/// Notice! In OpenGL matrices are column-major.
pub fn transform_quad(mat: &Matrix4, [a, b, c, d]: Quad) -> Quad {
    [transform_point(mat, a), transform_point(mat, b),
     transform_point(mat, c), transform_point(mat, d)]
}

/// Transform cube.
///
/// This transform is row-major, which is standard mathematical notation.
/// To convert column-major to row-major, use `mat4_transposed`.
///
/// Notice! In OpenGL matrices are column-major.
pub fn transform_cube(mat: &Matrix4, [a, b, c, d, e, f, g, h]: Cube) -> Cube {
    [
        transform_point(mat, a), transform_point(mat, b),
        transform_point(mat, c), transform_point(mat, d),
        transform_point(mat, e), transform_point(mat, f),
        transform_point(mat, g), transform_point(mat, h),
    ]
}

/// Transform triangle through model and view.
///
/// This transform is row-major, which is standard mathematical notation.
/// To convert column-major to row-major, use `mat4_transposed`.
///
/// Notice! In OpenGL matrices are column-major.
pub fn transform_view_model_triangle(
    view: &Matrix4,
    model: &Matrix4,
    tri: Triangle
) -> Triangle {
    transform_triangle(view, transform_triangle(model, tri))
}

/// Linear interpolate line using a parameter.
pub fn lerp_line((a, b): Line, t: f32) -> Point {
    use vecmath::vec3_add as add;
    use vecmath::vec3_sub as sub;
    use vecmath::vec3_scale as scale;

    add(a, scale(sub(b, a), t))
}

/// Linear interpolate RGB using a parameter.
pub fn lerp_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    use crate::color::{rgb_gamma_linear_to_srgb, rgb_gamma_srgb_to_linear};

    use vecmath::vec3_add as add;
    use vecmath::vec3_sub as sub;
    use vecmath::vec3_scale as scale;

    let a = rgb_gamma_srgb_to_linear(a);
    let b = rgb_gamma_srgb_to_linear(b);
    rgb_gamma_linear_to_srgb(add(a, scale(sub(b, a), t)))
}

/// Linear interpolate RGBA using a parameter.
pub fn lerp_rgba(a: Rgba, b: Rgba, t: f32) -> Rgba {
    use crate::color::{rgba_gamma_linear_to_srgb, rgba_gamma_srgb_to_linear};

    use vecmath::vec4_add as add;
    use vecmath::vec4_sub as sub;
    use vecmath::vec4_scale as scale;

    let a = rgba_gamma_srgb_to_linear(a);
    let b = rgba_gamma_srgb_to_linear(b);
    rgba_gamma_linear_to_srgb(add(a, scale(sub(b, a), t)))
}

/// Linear interpolate scalars using a parameter.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp to unit interval.
pub fn clamp(a: f32) -> f32 {if a >= 1.0 {1.0} else if a <= 0.0 {0.0} else {a}}
