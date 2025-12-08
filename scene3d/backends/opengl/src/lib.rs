
use vecmath::{
    Matrix4,
    Vector2,
    Vector3,
    col_mat4_mul,
    col_mat4_mul as mul,
    mat4_id,
};
use opengl_graphics::shader_utils::{
    compile_shader,
    uniform_location
};

use turbine_scene3d::*;

use std::path::Path;

/// Stores scene data.
pub struct Scene {
    /// Scene settings.
    pub settings: SceneSettings,
    /// Projection transform.
    pub projection: Matrix4<f32>,
    /// Camera transform.
    pub camera: Matrix4<f32>,
    /// Model transform.
    pub model: Matrix4<f32>,
    transform_stack: Vec<Matrix4<f32>>,
    shaders: Vec<gl::types::GLuint>,
    programs: Vec<gl::types::GLuint>,
    uniforms: Vec<gl::types::GLuint>,
    vertex_arrays: Vec<gl::types::GLuint>,
    buffers: Vec<gl::types::GLuint>,
    textures: Vec<gl::types::GLuint>,
}

impl Scene {
    /// Create new scene.
    pub fn new(settings: SceneSettings) -> Scene {
        let mat_id = mat4_id();
        Scene {
            settings,
            projection: mat_id,
            camera: mat_id,
            model: mat_id,
            shaders: vec![],
            programs: vec![],
            uniforms: vec![],
            vertex_arrays: vec![],
            buffers: vec![],
            transform_stack: vec![],
            textures: vec![],
        }
    }

    /// Set projection matrix.
    pub fn projection(&mut self, p: Matrix4<f32>) {
        self.projection = p;
    }

    /// Set camera matrix.
    pub fn camera(&mut self, c: Matrix4<f32>) {
        self.camera = c;
    }

    /// Set model matrix.
    pub fn model(&mut self, m: Matrix4<f32>) {
        self.model = m;
    }

    /// Load texture from path.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, image::ImageError> {
        use std::mem::transmute;

        let image = match image::open(path)? {
            image::DynamicImage::ImageRgba8(img) => img,
            x => x.to_rgba8()
        };
        let (image_width, image_height) = image.dimensions();
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
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
                gl::LINEAR as i32
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        let id = self.textures.len();
        self.textures.push(texture_id);
        Ok(Texture(id))
    }

    /// Create vertex shader from source.
    pub fn vertex_shader(
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
    pub fn fragment_shader(
        &mut self,
        fragment_shader_src: &str
    ) -> Result<FragmentShader, String> {
        let id = self.shaders.len();
        let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, fragment_shader_src)?;
        self.shaders.push(fragment_shader);
        Ok(FragmentShader(id))
    }

    /// Create program from vertex and fragment shader.
    pub fn program_from_vertex_fragment(
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

    /// Create 4D matrix uniform.
    pub fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(Matrix4Uniform(id))
    }

    /// Create 2D vector uniform.
    pub fn vector2_uniform(
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
    pub fn vector3_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector3Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(Vector3Uniform(id))
    }

    /// Create f32 uniform.
    pub fn f32_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<F32Uniform, String> {
        let id = self.uniforms.len();
        let uniform_location = uniform_location(self.programs[program.0], name)?;
        self.uniforms.push(uniform_location);
        Ok(F32Uniform(id))
    }

    /// Create vertex array.
    pub fn vertex_array(&mut self) -> VertexArray {
        unsafe {
            let id = self.vertex_arrays.len();
            let mut vertex_array_id = 0;
            gl::GenVertexArrays(1, &mut vertex_array_id);
            self.vertex_arrays.push(vertex_array_id);
            VertexArray(id)
        }
    }

    /// Create uv buffer.
    pub fn uv_buffer(
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

    /// Create color buffer.
    pub fn color_buffer(
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

    /// Create vertex buffer for 2D coordinates.
    pub fn vertex_buffer2(
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
    pub fn vertex_buffer3(
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

    /// Create normal buffer.
    pub fn normal_buffer(
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

    /// Use program.
    pub fn use_program(&self, program: Program) {
        unsafe {gl::UseProgram(self.programs[program.0])}
    }

    /// Set model-view-projection transform uniform.
    pub fn set_model_view_projection(&self, matrix_id: Matrix4Uniform) {
        unsafe {
            let mvp = mul(mul(self.projection, self.camera), self.model);
            gl::UniformMatrix4fv(self.uniforms[matrix_id.0] as i32, 1, gl::FALSE, &mvp[0][0])
        }
    }

    /// Set view transform uniform.
    pub fn set_view(&self, matrix_id: Matrix4Uniform) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms[matrix_id.0] as i32, 1, gl::FALSE, &self.camera[0][0])
        }
    }

    /// Set model transform uniform.
    pub fn set_model(&self, matrix_id: Matrix4Uniform) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms[matrix_id.0] as i32, 1, gl::FALSE, &self.model[0][0])
        }
    }

    /// Set matrix uniform.
    pub fn set_matrix4(&self, matrix_id: Matrix4Uniform, val: Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms[matrix_id.0] as i32, 1, gl::FALSE, &val[0][0])
        }
    }

    /// Set 2D vector uniform.
    pub fn set_vector2(&self, v_id: Vector2Uniform, v: Vector2<f32>) {
        unsafe {
            gl::Uniform2f(self.uniforms[v_id.0] as i32, v[0], v[1]);
        }
    }

    /// Set 3D vector uniform.
    pub fn set_vector3(&self, v_id: Vector3Uniform, v: Vector3<f32>) {
        unsafe {
            gl::Uniform3f(self.uniforms[v_id.0] as i32, v[0], v[1], v[2]);
        }
    }

    /// Set f32 uniform.
    pub fn set_f32(&self, f_id: F32Uniform, v: f32) {
        unsafe {
            gl::Uniform1f(self.uniforms[f_id.0] as i32, v);
        }
    }

    /// Set texture.
    pub fn set_texture(&self, texture_id: Texture) {
        unsafe {gl::BindTexture(gl::TEXTURE_2D, self.textures[texture_id.0])};
    }

    /// Translate model.
    pub fn translate(&mut self, v: Vector3<f32>) {
        let mat = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [v[0], v[1], v[2], 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Translate model in global coordinates.
    pub fn translate_global(&mut self, v: Vector3<f32>) {
        for i in 0..3 {
            self.model[3][i] += v[i];
        }
    }

    /// Scale model.
    pub fn scale(&mut self, v: Vector3<f32>) {
        let mat = [
            [v[0], 0.0, 0.0, 0.0],
            [0.0, v[1], 0.0, 0.0],
            [0.0, 0.0, v[2], 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Rotate model around x axis with degrees.
    pub fn rotate_x_deg(&mut self, deg: f32) {
        let angle = deg * 0.017453292519943295;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, sin, 0.0],
            [0.0, -sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Rotate model around y axis with degrees.
    pub fn rotate_y_deg(&mut self, deg: f32) {
        let angle = deg * 0.017453292519943295;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [cos, 0.0, sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Rotate model around z axis with degrees.
    pub fn rotate_z_deg(&mut self, deg: f32) {
        let angle = deg * 0.017453292519943295;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Rotate model around axis with degrees.
    pub fn rotate_axis_deg(&mut self, axis: Vector3<f32>, deg: f32) {
        let angle = deg * 0.017453292519943295;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [
                cos + axis[0]*axis[0]*(1.0-cos),
                axis[0]*axis[1]*(1.0-cos) + axis[2]*sin,
                axis[0]*axis[2]*(1.0-cos) - axis[1]*sin,
                0.0,
            ],
            [
                axis[0]*axis[1]*(1.0-cos) - axis[2]*sin,
                cos + axis[1]*axis[1]*(1.0-cos),
                axis[1]*axis[2]*(1.0-cos) + axis[0]*sin,
                0.0,
            ],
            [
                axis[0]*axis[2]*(1.0-cos) + axis[1]*sin,
                axis[1]*axis[2]*(1.0-cos) - axis[0]*sin,
                cos + axis[2]*axis[2]*(1.0-cos),
                0.0
            ],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = col_mat4_mul(self.model, mat);
    }

    /// Push model transfrom to transform stack.
    pub fn push_transform(&mut self) {
        self.transform_stack.push(self.model);
    }

    /// Pop model transform from transform stack.
    pub fn pop_transform(&mut self) {
        if let Some(mat) = self.transform_stack.pop() {
            self.model = mat;
        }
    }

    /// Clear background with color.
    pub fn clear(&self, bg_color: [f32; 4]) {
        unsafe {
            gl::ClearColor(bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
            if self.settings.clear_depth_buffer {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            } else {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            if self.settings.clear_enable_depth_test {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(gl::LESS);
            }
        }
    }

    /// Enable framebuffer sRGB.
    pub fn enable_framebuffer_srgb(&self) {
        unsafe {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }
    }

    /// Disable framebuffer sRGB.
    pub fn disable_framebuffer_srgb(&self) {
        unsafe {
            gl::Disable(gl::FRAMEBUFFER_SRGB);
        }
    }

    /// Enable blend.
    pub fn enable_blend(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
        }
    }

    /// Disable blend.
    pub fn disable_blend(&self) {
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    /// Enable cull face.
    pub fn enable_cull_face(&self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }
    }

    /// Disable cull face.
    pub fn disable_cull_face(&self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    /// Cull front face.
    pub fn cull_face_front(&self) {
        unsafe {
            gl::CullFace(gl::FRONT);
        }
    }

    /// Cull back face.
    pub fn cull_face_back(&self) {
        unsafe {
            gl::CullFace(gl::BACK);
        }
    }

    /// Cull both front and back face.
    pub fn cull_face_front_and_back(&self) {
        unsafe {
            gl::CullFace(gl::FRONT_AND_BACK);
        }
    }

    /// Draws triangles.
    pub fn draw_triangles(&self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::TRIANGLES, 0, len as i32);
        }
    }

    /// Draws triangle strip.
    pub fn draw_triangle_strip(&self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, len as i32);
        }
    }

    /// Draws points.
    pub fn draw_points(&self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::POINTS, 0, len as i32);
        }
    }

    /// Draws lines.
    pub fn draw_lines(&self, vertex_array: VertexArray, len: usize) {
        unsafe {
            gl::BindVertexArray(self.vertex_arrays[vertex_array.0]);
            gl::DrawArrays(gl::LINES, 0, len as i32);
        }
    }

    /// Executes commands in command list.
    pub fn submit(&mut self, commands: &[Command], frame_graph: &FrameGraph) {
        use Command::*;

        for command in commands {
            match *command {
                UseProgram(program) => self.use_program(program),
                SetModelViewProjection(mvp) => self.set_model_view_projection(mvp),
                SetModel(model) => self.set_model(model),
                SetView(view) => self.set_view(view),
                SetTexture(texture) => self.set_texture(texture),
                SetF32(uni, val) => self.set_f32(uni, val),
                SetVector2(uni, val) => self.set_vector2(uni, val),
                SetVector3(uni, val) => self.set_vector3(uni, val),
                SetMatrix4(uni, val) => self.set_matrix4(uni, val),
                EnableFrameBufferSRGB => self.enable_framebuffer_srgb(),
                DisableFrameBufferSRGB => self.disable_framebuffer_srgb(),
                EnableBlend => self.enable_blend(),
                DisableBlend => self.disable_blend(),
                EnableCullFace => self.enable_cull_face(),
                DisableCullFace => self.disable_cull_face(),
                CullFaceFront => self.cull_face_front(),
                CullFaceBack => self.cull_face_back(),
                CullFaceFrontAndBack => self.cull_face_front_and_back(),
                DrawTriangles(vertex_array, len) => self.draw_triangles(vertex_array, len),
                DrawTriangleStrip(vertex_array, len) => self.draw_triangle_strip(vertex_array, len),
                DrawLines(vertex_array, len) => self.draw_lines(vertex_array, len),
                DrawPoints(vertex_array, len) => self.draw_points(vertex_array, len),
                Translate(v) => self.translate(v),
                TranslateGlobal(v) => self.translate_global(v),
                Scale(v) => self.scale(v),
                RotateXDeg(deg) => self.rotate_x_deg(deg),
                RotateYDeg(deg) => self.rotate_y_deg(deg),
                RotateZDeg(deg) => self.rotate_z_deg(deg),
                RotateAxisDeg(axis, deg) => self.rotate_axis_deg(axis, deg),
                PushTransform => self.push_transform(),
                PopTransform => self.pop_transform(),
                Draw(command_list) => self.draw(command_list, frame_graph),
            }
        }
    }

    /// Draws a command list from frame graph.
    pub fn draw(&mut self, command_list: CommandList, frame_graph: &FrameGraph) {
        self.submit(&frame_graph.command_lists[command_list.0], frame_graph)
    }
}

impl Drop for Scene {
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
