//! Types.

use gfx_device_gl;
use gfx_debug_draw;

/// The type of debug renderer.
pub type DebugRenderer =
    gfx_debug_draw::DebugRenderer<
        gfx_device_gl::Resources, gfx_device_gl::Factory>;
