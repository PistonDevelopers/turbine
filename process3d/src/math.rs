//! Mathematical algorithms.

pub use vecmath::mat4_id;

use crate::{Line, Matrix4, Point, Triangle};

/// Transform point.
pub fn transform_point(mat: &Matrix4, p: Point) -> Point {
    [
        mat[0][0] * p[0] + mat[0][1] * p[1] + mat[0][2] * p[2] + mat[0][3],
        mat[1][0] * p[0] + mat[1][1] * p[1] + mat[1][2] * p[2] + mat[1][3],
        mat[2][0] * p[0] + mat[2][1] * p[1] + mat[2][2] * p[2] + mat[2][3],
    ]
}

/// Transform triangle.
pub fn transform_triangle(mat: &Matrix4, (a, b, c): Triangle) -> Triangle {
    (transform_point(mat, a), transform_point(mat, b), transform_point(mat, c))
}

/// Transform triangle through model and view.
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
