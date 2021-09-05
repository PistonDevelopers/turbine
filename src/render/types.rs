//! Types.

use gfx_debug_draw;
use gfx_device_gl;

/// The type of debug renderer.
pub type DebugRenderer =
    gfx_debug_draw::DebugRenderer<gfx_device_gl::Resources, gfx_device_gl::Factory>;
