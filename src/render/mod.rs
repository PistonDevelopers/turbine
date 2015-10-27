//! Rendering.

use self::types::Stream;
use self::types::DebugRenderer;
use camera_controllers::Camera;

use math::Mat4;

pub mod types;

/// Clears the screen.
pub fn clear(stream: &mut Stream) {
    use gfx::ClearData;
    use gfx::traits::*;

    stream.clear(
        ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        }
    );
}

/// Draws axes.
pub fn axes(
    camera: &Camera,
    projection: Mat4,
    draw_size: [u32; 2],
    debug_renderer: &mut DebugRenderer
) {
    use math::is_looking_in_direction_of;
    use math::{ Matrix, Vector };
    use camera_controllers::model_view_projection;

    let red = [1.0, 0.0, 0.0, 1.0];
    let green = [0.0, 1.0, 0.0, 1.0];
    let blue = [0.0, 0.0, 1.0, 1.0];
    let origo = [0.0, 0.0, 0.0];
    let x = [1.0, 0.0, 0.0];
    let y = [0.0, 1.0, 0.0];
    let z = [0.0, 0.0, 1.0];
    debug_renderer.draw_line(origo, x, red);
    debug_renderer.draw_line(origo, y, green);
    debug_renderer.draw_line(origo, z, blue);

    let mvp = model_view_projection(
        Matrix::id(),
        camera.orthogonal(),
        projection
    );

    if is_looking_in_direction_of(camera.position, camera.forward, x) {
        let x = mvp.pos_to_frame_buffer(x, draw_size).i32x2();
        debug_renderer.draw_text_on_screen("X", x, red);
    }
    if is_looking_in_direction_of(camera.position, camera.forward, y) {
        let y = mvp.pos_to_frame_buffer(y, draw_size).i32x2();
        debug_renderer.draw_text_on_screen("Y", y, green);
    }
    if is_looking_in_direction_of(camera.position, camera.forward, z) {
        let z = mvp.pos_to_frame_buffer(z, draw_size).i32x2();
        debug_renderer.draw_text_on_screen("Z", z, blue);
    }
}
