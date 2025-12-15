#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use vecmath::{
    Matrix4,
    Vector2,
    Vector3,
};
use opengl_graphics::TextureSettings;
use opengl_graphics::shader_utils::{
    compile_shader,
    uniform_location,
};

use turbine_scene3d::*;

use std::path::Path;

mod gl_utils;

/// Implements OpenGL backend.
pub struct State {
    shaders: Vec<gl::types::GLuint>,
    programs: Vec<gl::types::GLuint>,
    uniforms: Vec<gl::types::GLuint>,
    vertex_arrays: Vec<gl::types::GLuint>,
    buffers: Vec<gl::types::GLuint>,
    textures: Vec<gl::types::GLuint>,
}

impl State {
    /// Creates a new backend state.
    pub fn new() -> State {
        State {
            shaders: vec![],
            programs: vec![],
            uniforms: vec![],
            vertex_arrays: vec![],
            buffers: vec![],
            textures: vec![],
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            for &shader in &self.shaders {
                gl::DeleteShader(shader);
            }
            for &program in &self.programs {
                gl::DeleteProgram(program);
            }
            if self.buffers.len() > 0 {
                gl::DeleteBuffers(self.buffers.len() as i32, &self.buffers[0]);
            }
            if self.textures.len() > 0 {
                gl::DeleteTextures(self.textures.len() as i32, &self.textures[0]);
            }
            if self.vertex_arrays.len() > 0 {
                gl::DeleteVertexArrays(self.vertex_arrays.len() as i32, &self.vertex_arrays[0]);
            }
        }
    }
}

impl Backend for State {
    type ImageError = image::ImageError;

    /// Enable framebuffer sRGB.
    fn enable_framebuffer_srgb(&mut self) {
        unsafe {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }
    }

    /// Disable framebuffer sRGB.
    fn disable_framebuffer_srgb(&mut self) {
        unsafe {
            gl::Disable(gl::FRAMEBUFFER_SRGB);
        }
    }

    /// Enable blend.
    fn enable_blend(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
        }
    }

    /// Disable blend.
    fn disable_blend(&mut self) {
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    /// Use program.
    fn use_program(&mut self, program: Program) {
        unsafe {gl::UseProgram(self.programs[program.0])}
    }

    /// Set matrix uniform.
    fn set_matrix4(&mut self, matrix_id: Matrix4Uniform, val: Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms[matrix_id.0] as i32, 1, gl::FALSE, &val[0][0])
        }
    }

    /// Set 2D vector uniform.
    fn set_vector2(&mut self, v_id: Vector2Uniform, v: Vector2<f32>) {
        unsafe {
            gl::Uniform2f(self.uniforms[v_id.0] as i32, v[0], v[1]);
        }
    }

    /// Set 3D vector uniform.
    fn set_vector3(&mut self, v_id: Vector3Uniform, v: Vector3<f32>) {
        unsafe {
            gl::Uniform3f(self.uniforms[v_id.0] as i32, v[0], v[1], v[2]);
        }
    }

    /// Set f32 uniform.
    fn set_f32(&mut self, f_id: F32Uniform, v: f32) {
        unsafe {
            gl::Uniform1f(self.uniforms[f_id.0] as i32, v);
        }
    }

    /// Set texture.
    fn set_texture(&mut self, texture_id: Texture) {
        unsafe {gl::BindTexture(gl::TEXTURE_2D, self.textures[texture_id.0])};
    }

    /// Enable cull face.
    fn enable_cull_face(&mut self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }
    }

    /// Disable cull face.
    fn disable_cull_face(&mut self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    /// Cull front face.
    fn cull_face_front(&mut self) {
        unsafe {
            gl::CullFace(gl::FRONT);
        }
    }

    /// Cull back face.
    fn cull_face_back(&mut self) {
        unsafe {
            gl::CullFace(gl::BACK);
        }
    }

    /// Cull both front and back face.
    fn cull_face_front_and_back(&mut self) {
        unsafe {
            gl::CullFace(gl::FRONT_AND_BACK);
        }
    }

    /// Draws triangles.
    fn draw_triangles(&mut self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::TRIANGLES, 0, len as i32);
        }
    }

    /// Draws triangle strip.
    fn draw_triangle_strip(&mut self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, len as i32);
        }
    }

    /// Draws points.
    fn draw_points(&mut self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::POINTS, 0, len as i32);
        }
    }

    /// Draws lines.
    fn draw_lines(&mut self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::LINES, 0, len as i32);
        }
    }

    /// Clear background with color.
    fn clear(&mut self, bg_color: [f32; 4], settings: &SceneSettings) {
        unsafe {
            gl::ClearColor(bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
            if settings.clear_depth_buffer {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            } else {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            if settings.clear_enable_depth_test {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(gl::LESS);
            }
        }
    }

    /// Create vertex shader from source.
    fn vertex_shader(
        &mut self,
        vertex_shader_src: &str
    ) -> Result<VertexShader, String> {
        let id = self.shaders.len();
        let vertex_shader = compile_shader(gl::VERTEX_SHADER, vertex_shader_src)
            .unwrap();
        self.shaders.push(vertex_shader);
        Ok(VertexShader(id))
    }

    /// Create fragment shader from source.
    fn fragment_shader(
        &mut self,
        fragment_shader_src: &str
    ) -> Result<FragmentShader, String> {
        let id = self.shaders.len();
        let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, fragment_shader_src)?;
        self.shaders.push(fragment_shader);
        Ok(FragmentShader(id))
    }

    /// Create vertex buffer for 2D coordinates.
    fn vertex_buffer2(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer2 {
        use std::mem::{size_of, transmute};
        use std::ptr::null;

        unsafe {
            let id = self.buffers.len();
            let mut vertex_buffer = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<gl::types::GLfloat>()) as isize,
                transmute(data.as_ptr()),
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::VertexAttribPointer(
                attribute,  // attribute
                2,          // size
                gl::FLOAT,  // type
                gl::FALSE,  // normalized?
                0,          // stride
                null()      // array buffer offset
            );
            gl::EnableVertexAttribArray(attribute);

            self.buffers.push(vertex_buffer);
            VertexBuffer2(id, data.len() / 2)
        }
    }

    /// Create vertex buffer for 3D coordinates.
    fn vertex_buffer3(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer3 {
        use std::mem::{size_of, transmute};
        use std::ptr::null;

        unsafe {
            let id = self.buffers.len();
            let mut vertex_buffer = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<gl::types::GLfloat>()) as isize,
                transmute(data.as_ptr()),
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::VertexAttribPointer(
                attribute,  // attribute
                3,          // size
                gl::FLOAT,  // type
                gl::FALSE,  // normalized?
                0,          // stride
                null()      // array buffer offset
            );
            gl::EnableVertexAttribArray(attribute);

            self.buffers.push(vertex_buffer);
            VertexBuffer3(id, data.len() / 3)
        }
    }

    /// Create color buffer.
    fn color_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> ColorBuffer {
        use std::mem::{size_of, transmute};
        use std::ptr::null;

        unsafe {
            let id = self.buffers.len();
            let mut color_buffer = 0;
            gl::GenBuffers(1, &mut color_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<gl::types::GLfloat>()) as isize,
                transmute(data.as_ptr()),
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::VertexAttribPointer(
                attribute,  // attribute
                3,          // size
                gl::FLOAT,  // type
                gl::FALSE,  // normalized?
                0,          // stride
                null()      // array buffer offset
            );
            gl::EnableVertexAttribArray(attribute);

            self.buffers.push(color_buffer);
            ColorBuffer(id, data.len() / 3)
        }
    }

    /// Create vertex array.
    fn vertex_array(&mut self) -> VertexArray {
        unsafe {
            let id = self.vertex_arrays.len();
            let mut vertex_array_id = 0;
            gl::GenVertexArrays(1, &mut vertex_array_id);
            self.vertex_arrays.push(vertex_array_id);
            VertexArray(id)
        }
    }

    /// Create 4D matrix uniform.
    fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(Matrix4Uniform(id))
    }

    /// Create program from vertex and fragment shader.
    fn program_from_vertex_fragment(
        &mut self,
        vertex_shader: VertexShader,
        fragment_shader: FragmentShader
    ) -> Program {
        unsafe {
            let id = self.programs.len();
            let program = gl::CreateProgram();
            gl::AttachShader(program, self.shaders[vertex_shader.0]);
            gl::AttachShader(program, self.shaders[fragment_shader.0]);
            gl::LinkProgram(program);
            gl::DetachShader(program, self.shaders[vertex_shader.0]);
            gl::DetachShader(program, self.shaders[fragment_shader.0]);
            self.programs.push(program);
            Program(id)
        }
    }

    /// Create f32 uniform.
    fn f32_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<F32Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(F32Uniform(id))
    }

    /// Create 2D vector uniform.
    fn vector2_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector2Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(Vector2Uniform(id))
    }

    /// Create 3D vector uniform.
    fn vector3_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector3Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(Vector3Uniform(id))
    }

    /// Load texture from path.
    fn load_texture<P: AsRef<Path>>(
        &mut self,
        path: P,
        settings: &TextureSettings
    ) -> Result<Texture, image::ImageError> {
        use std::mem::transmute;
        use gl_utils::GlSettings;
        use opengl_graphics::Wrap;

        let image = match image::open(path)? {
            image::DynamicImage::ImageRgba8(img) => img,
            x => x.to_rgba8()
        };
        let (image_width, image_height) = image.dimensions();
        let mut texture_id = 0;
        let internal_format = if settings.get_convert_gamma() {
            gl::RGBA
        } else {
            gl::SRGB_ALPHA
        };
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                image_width as i32,
                image_height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                transmute(image.as_ptr())
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                settings.get_gl_mag() as i32
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                settings.get_gl_min() as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                settings.get_gl_wrap_u() as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                settings.get_gl_wrap_v() as i32,
            );
            if settings.get_wrap_u() == Wrap::ClampToBorder
                || settings.get_wrap_v() == Wrap::ClampToBorder
            {
                gl::TexParameterfv(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_BORDER_COLOR,
                    settings.get_border_color().as_ptr(),
                );
            }
            if settings.get_generate_mipmap() {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
        }
        let id = self.textures.len();
        self.textures.push(texture_id);
        Ok(Texture(id))
    }

    /// Create normal buffer.
    fn normal_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> NormalBuffer {
        use std::mem::{size_of, transmute};
        use std::ptr::null;

        unsafe {
            let id = self.buffers.len();
            let mut vertex_buffer = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<gl::types::GLfloat>()) as isize,
                transmute(data.as_ptr()),
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::VertexAttribPointer(
                attribute,  // attribute
                3,          // size
                gl::FLOAT,  // type
                gl::FALSE,  // normalized?
                0,          // stride
                null()      // array buffer offset
            );
            gl::EnableVertexAttribArray(attribute);

            self.buffers.push(vertex_buffer);
            NormalBuffer(id, data.len() / 3)
        }
    }

    /// Create uv buffer.
    fn uv_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> UVBuffer {
        use std::mem::{size_of, transmute};
        use std::ptr::null;

        unsafe {
            let id = self.buffers.len();
            let mut color_buffer = 0;
            gl::GenBuffers(1, &mut color_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<gl::types::GLfloat>()) as isize,
                transmute(data.as_ptr()),
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::VertexAttribPointer(
                attribute,  // attribute
                2,          // size
                gl::FLOAT,  // type
                gl::FALSE,  // normalized?
                0,          // stride
                null()      // array buffer offset
            );
            gl::EnableVertexAttribArray(attribute);

            self.buffers.push(color_buffer);
            UVBuffer(id, data.len() / 2)
        }
    }
}
