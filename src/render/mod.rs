//! Rendering.

use self::types::DebugRenderer;
use camera_controllers::Camera;

use crate::math::Mat4;
use crate::world::World;

pub mod types;

/// Draws axes.
pub fn axes(
    camera: &Camera,
    projection: Mat4,
    draw_size: [u32; 2],
    debug_renderer: &mut DebugRenderer,
) {
    use crate::math::is_looking_in_direction_of;
    use crate::math::{Matrix, Vector};
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

    let mvp = model_view_projection(Matrix::id(), camera.orthogonal(), projection);

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

/// Renders the current positions of entities.
pub fn entity_current_positions(world: &World, debug_renderer: &mut DebugRenderer) {
    use crate::world::*;

    let turqouise = [0.0, 1.0, 1.0, 1.0];
    for i in 0..ENTITY_COUNT {
        if world.mask[i].contains(Mask::ALIVE) {
            debug_renderer.draw_marker(world.current.position[i], 0.1, turqouise);
        }
    }
}
