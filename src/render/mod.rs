//! Rendering.

use self::types::Stream;
use self::types::DebugRenderer;
use camera_controllers::Camera;

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
pub fn axes(camera: &Camera, debug_renderer: &mut DebugRenderer) {
    use math::is_looking_in_direction_of;

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
    if is_looking_in_direction_of(camera.position, camera.forward, x) {
        debug_renderer.draw_text_at_position("X", x, red);
    }
    if is_looking_in_direction_of(camera.position, camera.forward, y) {
        debug_renderer.draw_text_at_position("Y", y, green);
    }
    if is_looking_in_direction_of(camera.position, camera.forward, z) {
        debug_renderer.draw_text_at_position("Z", z, blue);
    }
}
