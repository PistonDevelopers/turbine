//! Rendering.

use self::types::Stream;
use self::types::DebugRenderer;

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
pub fn axes(debug_renderer: &mut DebugRenderer) {
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
    debug_renderer.draw_text_at_position("X", x, red);
    debug_renderer.draw_text_at_position("Y", y, green);
    debug_renderer.draw_text_at_position("Z", z, blue);
}
