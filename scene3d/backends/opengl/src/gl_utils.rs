use opengl_graphics::{Filter, TextureSettings, Wrap};

pub(crate) trait GlSettings {
    fn get_gl_mag(&self) -> gl::types::GLenum;
    fn get_gl_min(&self) -> gl::types::GLenum;
    #[allow(dead_code)]
    fn get_gl_mipmap(&self) -> gl::types::GLenum;
    fn get_gl_wrap_u(&self) -> gl::types::GLenum;
    fn get_gl_wrap_v(&self) -> gl::types::GLenum;
}

impl GlSettings for TextureSettings {
    fn get_gl_mag(&self) -> gl::types::GLenum {
        match self.get_mag() {
            Filter::Linear => gl::LINEAR,
            Filter::Nearest => gl::NEAREST,
        }
    }

    fn get_gl_min(&self) -> gl::types::GLenum {
        match self.get_min() {
            Filter::Linear => {
                if self.get_generate_mipmap() {
                    match self.get_mipmap() {
                        Filter::Linear => gl::LINEAR_MIPMAP_LINEAR,
                        Filter::Nearest => gl::LINEAR_MIPMAP_NEAREST,
                    }
                } else {
                    gl::LINEAR
                }
            }
            Filter::Nearest => {
                if self.get_generate_mipmap() {
                    match self.get_mipmap() {
                        Filter::Linear => gl::NEAREST_MIPMAP_LINEAR,
                        Filter::Nearest => gl::NEAREST_MIPMAP_NEAREST,
                    }
                } else {
                    gl::NEAREST
                }
            }
        }
    }

    fn get_gl_mipmap(&self) -> gl::types::GLenum {
        match self.get_mipmap() {
            Filter::Linear => gl::LINEAR,
            Filter::Nearest => gl::NEAREST,
        }
    }

    fn get_gl_wrap_u(&self) -> gl::types::GLenum {
        match self.get_wrap_u() {
            Wrap::Repeat => gl::REPEAT,
            Wrap::MirroredRepeat => gl::MIRRORED_REPEAT,
            Wrap::ClampToEdge => gl::CLAMP_TO_EDGE,
            Wrap::ClampToBorder => gl::CLAMP_TO_BORDER,
        }
    }

    fn get_gl_wrap_v(&self) -> gl::types::GLenum {
        match self.get_wrap_v() {
            Wrap::Repeat => gl::REPEAT,
            Wrap::MirroredRepeat => gl::MIRRORED_REPEAT,
            Wrap::ClampToEdge => gl::CLAMP_TO_EDGE,
            Wrap::ClampToBorder => gl::CLAMP_TO_BORDER,
        }
    }
}
