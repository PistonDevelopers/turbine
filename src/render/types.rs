//! Types.

use gfx;
use gfx_device_gl;
use gfx_debug_draw;

/// The type of Gfx stream.
pub type Stream =
    gfx::OwnedStream<gfx_device_gl::Device, gfx_device_gl::Output>;

/// The type of debug renderer.
pub type DebugRenderer =
    gfx_debug_draw::DebugRenderer<
        gfx_device_gl::Resources, gfx_device_gl::Factory>;
