//! # Scene

use std::path::Path;
use vecmath::{Vector2, Vector3, Matrix4};
use crate::{
    ColorBuffer,
    Command,
    CommandList,
    External,
    F32Uniform,
    FragmentShader,
    FrameGraph,
    Matrix4Uniform,
    NormalBuffer,
    Program,
    Texture,
    UVBuffer,
    Vector2Uniform,
    Vector3Uniform,
    VertexArray,
    VertexBuffer2,
    VertexBuffer3,
    VertexShader,
};


/// Stores scene settings.
#[derive(Clone)]
pub struct SceneSettings {
    /// Whether to clear depth buffer.
    pub clear_depth_buffer: bool,
    /// Whether to enable depth test.
    pub clear_enable_depth_test: bool,
}

impl Default for SceneSettings {
    fn default() -> Self {SceneSettings::new()}
}

impl SceneSettings {
    /// Returns new scene settings with default settings.
    pub fn new() -> SceneSettings {
        SceneSettings {
            clear_depth_buffer: true,
            clear_enable_depth_test: true,
        }
    }

    /// Set whether to clear depth buffer on clear.
    pub fn clear_depth_buffer(mut self, val: bool) -> Self {
        self.clear_depth_buffer = val;
        self
    }

    /// Set whether to enable depth test on clear.
    ///
    /// Uses depth test function `LESS` by default.
    pub fn clear_enable_depth_test(mut self, val: bool) -> Self {
        self.clear_enable_depth_test = val;
        self
    }
}

/// Implemented by Scene backend states.
pub trait Backend {
    /// The image error type.
    type ImageError;

    /// Set texture.
    fn set_texture(&self, texture_id: Texture);
    /// Draws points.
    fn draw_points(&self, vertex_array: VertexArray, len: usize);
    /// Draws lines.
    fn draw_lines(&self, vertex_array: VertexArray, len: usize);
    /// Draws triangle strip.
    fn draw_triangle_strip(&self, vertex_array: VertexArray, len: usize);
    /// Draws triangles.
    fn draw_triangles(&self, vertex_array: VertexArray, len: usize);
    /// Enable framebuffer sRGB.
    fn enable_framebuffer_srgb(&self);
    /// Disable framebuffer sRGB.
    fn disable_framebuffer_srgb(&self);
    /// Enable blend.
    fn enable_blend(&self);
    /// Disable blend.
    fn disable_blend(&self);
    /// Enable cull face.
    fn enable_cull_face(&self);
    /// Disable cull face.
    fn disable_cull_face(&self);
    /// Cull front face.
    fn cull_face_front(&self);
    /// Cull back face.
    fn cull_face_back(&self);
    /// Cull both front and back face.
    fn cull_face_front_and_back(&self);
    /// Clear background with color.
    fn clear(&self, bg_color: [f32; 4], settings: &SceneSettings);
    /// Set f32 uniform.
    fn set_f32(&self, f_id: F32Uniform, v: f32);
    /// Set 2D vector uniform.
    fn set_vector2(&self, v_id: Vector2Uniform, v: Vector2<f32>);
    /// Set 3D vector uniform.
    fn set_vector3(&self, v_id: Vector3Uniform, v: Vector3<f32>);
    /// Set matrix uniform.
    fn set_matrix4(&self, matrix_id: Matrix4Uniform, val: Matrix4<f32>);
    /// Use program.
    fn use_program(&self, program: Program);
    /// Create vertex shader from source.
    fn vertex_shader(
        &mut self,
        vertex_shader_src: &str
    ) -> Result<VertexShader, String>;
    /// Create fragment shader from source.
    fn fragment_shader(
        &mut self,
        fragment_shader_src: &str
    ) -> Result<FragmentShader, String>;
    /// Create vertex buffer for 2D coordinates.
    fn vertex_buffer2(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer2;
    /// Create vertex buffer for 3D coordinates.
    fn vertex_buffer3(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer3;
    /// Create color buffer.
    fn color_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> ColorBuffer;
    /// Create vertex array.
    fn vertex_array(&mut self) -> VertexArray;
    /// Create 4D matrix uniform.
    fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String>;
    /// Create program from vertex and fragment shader.
    fn program_from_vertex_fragment(
        &mut self,
        vertex_shader: VertexShader,
        fragment_shader: FragmentShader
    ) -> Program;
    /// Create f32 uniform.
    fn f32_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<F32Uniform, String>;
    /// Create 2D vector uniform.
    fn vector2_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector2Uniform, String>;
    /// Create 3D vector uniform.
    fn vector3_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector3Uniform, String>;
    /// Load texture from path.
    fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, Self::ImageError>;
    /// Create normal buffer.
    fn normal_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> NormalBuffer;
    /// Create uv buffer.
    fn uv_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> UVBuffer;
}

/// Stores scene data.
pub struct Scene<State> {
    /// The state of the scene backend.
    pub state: State,
    /// Scene settings.
    pub settings: SceneSettings,
    /// Projection transform.
    pub projection: Matrix4<f32>,
    /// Camera transform.
    pub camera: Matrix4<f32>,
    /// Model transform.
    pub model: Matrix4<f32>,
    transform_stack: Vec<Matrix4<f32>>,
    externals: Vec<fn(&mut Self)>,
}

const DEG_TO_RAD: f32 = 0.017453292519943295;

impl<State> Scene<State>
    where State: Backend
{
    /// Create new scene.
    pub fn new(settings: SceneSettings, state: State) -> Self {
        let mat_id = vecmath::mat4_id();
        Scene {
            state,
            settings,
            projection: mat_id,
            camera: mat_id,
            model: mat_id,
            transform_stack: vec![],
            externals: vec![],
        }
    }

    /// Add external action.
    pub fn external(&mut self, f: fn(&mut Self)) -> External {
        let id = self.externals.len();
        self.externals.push(f);
        External(id)
    }

    /// Call external action.
    pub fn call_external(&mut self, id: External) {
        let f = self.externals[id.0];
        (f)(self);
    }

    /// Translate model.
    pub fn translate(&mut self, v: Vector3<f32>) {
        let mat = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [v[0], v[1], v[2], 1.0]
        ];
        self.model = vecmath::col_mat4_mul(self.model, mat);
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
        self.model = vecmath::col_mat4_mul(self.model, mat);
    }

    /// Rotate model around x axis with degrees.
    pub fn rotate_x_deg(&mut self, deg: f32) {
        let angle = deg * DEG_TO_RAD;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, sin, 0.0],
            [0.0, -sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = vecmath::col_mat4_mul(self.model, mat);
    }

    /// Rotate model around y axis with degrees.
    pub fn rotate_y_deg(&mut self, deg: f32) {
        let angle = deg * DEG_TO_RAD;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [cos, 0.0, sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = vecmath::col_mat4_mul(self.model, mat);
    }

    /// Rotate model around z axis with degrees.
    pub fn rotate_z_deg(&mut self, deg: f32) {
        let angle = deg * DEG_TO_RAD;
        let cos = angle.cos();
        let sin = angle.sin();
        let mat = [
            [cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        self.model = vecmath::col_mat4_mul(self.model, mat);
    }

    /// Rotate model around axis with degrees.
    pub fn rotate_axis_deg(&mut self, axis: Vector3<f32>, deg: f32) {
        let angle = deg * DEG_TO_RAD;
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
        self.model = vecmath::col_mat4_mul(self.model, mat);
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

    /// Draws a command list from frame graph.
    pub fn draw(&mut self, command_list: CommandList, frame_graph: &FrameGraph) {
        self.submit(&frame_graph.command_lists[command_list.0], frame_graph)
    }

    /// Set model-view-projection transform uniform.
    pub fn set_model_view_projection(&self, matrix_id: Matrix4Uniform) {
        use vecmath::col_mat4_mul as mul;
        let mvp = mul(mul(self.projection, self.camera), self.model);
        self.state.set_matrix4(matrix_id, mvp);
    }

    /// Set view transform uniform.
    pub fn set_view(&self, matrix_id: Matrix4Uniform) {
        self.state.set_matrix4(matrix_id, self.camera);
    }

    /// Set model transform uniform.
    pub fn set_model(&self, matrix_id: Matrix4Uniform) {
        self.state.set_matrix4(matrix_id, self.model);
    }

    /// Create vertex shader from source.
    pub fn vertex_shader(
        &mut self,
        vertex_shader_src: &str
    ) -> Result<VertexShader, String> {
        self.state.vertex_shader(vertex_shader_src)
    }

    /// Create fragment shader from source.
    pub fn fragment_shader(
        &mut self,
        fragment_shader_src: &str
    ) -> Result<FragmentShader, String> {
        self.state.fragment_shader(fragment_shader_src)
    }

    /// Clear background with color.
    pub fn clear(&self, bg_color: [f32; 4]) {
        self.state.clear(bg_color, &self.settings);
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

    /// Create vertex buffer for 2D coordinates.
    pub fn vertex_buffer2(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer2 {
        self.state.vertex_buffer2(vertex_array, attribute, data)
    }

    /// Create vertex buffer for 3D coordinates.
    pub fn vertex_buffer3(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer3 {
        self.state.vertex_buffer3(vertex_array, attribute, data)
    }

    /// Create color buffer.
    pub fn color_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> ColorBuffer {
        self.state.color_buffer(vertex_array, attribute, data)
    }

    /// Create vertex array.
    pub fn vertex_array(&mut self) -> VertexArray {
        self.state.vertex_array()
    }

    /// Create 4D matrix uniform.
    pub fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String> {
        self.state.matrix4_uniform(program, name)
    }

    /// Create program from vertex and fragment shader.
    pub fn program_from_vertex_fragment(
        &mut self,
        vertex_shader: VertexShader,
        fragment_shader: FragmentShader
    ) -> Program {
        self.state.program_from_vertex_fragment(vertex_shader, fragment_shader)
    }

    /// Set f32 uniform.
    pub fn set_f32(&self, f_id: F32Uniform, v: f32) {
        self.state.set_f32(f_id, v)
    }

    /// Set 2D vector uniform.
    pub fn set_vector2(&self, v_id: Vector2Uniform, v: Vector2<f32>) {
        self.state.set_vector2(v_id, v)
    }

    /// Set 3D vector uniform.
    pub fn set_vector3(&self, v_id: Vector3Uniform, v: Vector3<f32>) {
        self.state.set_vector3(v_id, v)
    }

    /// Set matrix uniform.
    pub fn set_matrix4(&self, matrix_id: Matrix4Uniform, val: Matrix4<f32>) {
        self.state.set_matrix4(matrix_id, val)
    }

    /// Set texture.
    pub fn set_texture(&self, texture_id: Texture) {
        self.state.set_texture(texture_id)
    }

    /// Use program.
    pub fn use_program(&self, program: Program) {
        self.state.use_program(program)
    }

    /// Create f32 uniform.
    pub fn f32_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<F32Uniform, String> {
        self.state.f32_uniform(program, name)
    }

    /// Create 2D vector uniform.
    pub fn vector2_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector2Uniform, String> {
        self.state.vector2_uniform(program, name)
    }

    /// Create 3D vector uniform.
    pub fn vector3_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector3Uniform, String> {
        self.state.vector3_uniform(program, name)
    }

    /// Load texture from path.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, State::ImageError> {
        self.state.load_texture(path)
    }

    /// Create normal buffer.
    pub fn normal_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> NormalBuffer {
        self.state.normal_buffer(vertex_array, attribute, data)
    }

    /// Create uv buffer.
    pub fn uv_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> UVBuffer {
        self.state.uv_buffer(vertex_array, attribute, data)
    }

    /// Enable framebuffer sRGB.
    pub fn enable_framebuffer_srgb(&self) {
        self.state.enable_framebuffer_srgb()
    }

    /// Disable framebuffer sRGB.
    pub fn disable_framebuffer_srgb(&self) {
        self.state.disable_framebuffer_srgb()
    }

    /// Enable blend.
    pub fn enable_blend(&self) {
        self.state.enable_blend()
    }

    /// Disable blend.
    pub fn disable_blend(&self) {
        self.state.disable_blend()
    }

    /// Enable cull face.
    pub fn enable_cull_face(&self) {
        self.state.enable_cull_face()
    }

    /// Disable cull face.
    pub fn disable_cull_face(&self) {
        self.state.disable_cull_face()
    }

    /// Cull front face.
    pub fn cull_face_front(&self) {
        self.state.cull_face_front()
    }

    /// Cull back face.
    pub fn cull_face_back(&self) {
        self.state.cull_face_back()
    }

    /// Cull both front and back face.
    pub fn cull_face_front_and_back(&self) {
        self.state.cull_face_front_and_back()
    }

    /// Draws triangles.
    pub fn draw_triangles(&self, vertex_array: VertexArray, len: usize) {
        self.state.draw_triangles(vertex_array, len)
    }

    /// Draws triangle strip.
    pub fn draw_triangle_strip(&self, vertex_array: VertexArray, len: usize) {
        self.state.draw_triangle_strip(vertex_array, len)
    }

    /// Draws points.
    pub fn draw_points(&self, vertex_array: VertexArray, len: usize) {
        self.state.draw_points(vertex_array, len)
    }

    /// Draws lines.
    pub fn draw_lines(&self, vertex_array: VertexArray, len: usize) {
        self.state.draw_lines(vertex_array, len)
    }

    /// Executes commands in command list.
    pub fn submit(&mut self, commands: &[Command], frame_graph: &FrameGraph) {
        use crate::Command::*;

        for command in commands {
            match *command {
                CallExternal(external) => self.call_external(external),
                UseProgram(program) => self.state.use_program(program),
                SetModelViewProjection(mvp) => self.set_model_view_projection(mvp),
                SetModel(model) => self.set_model(model),
                SetView(view) => self.set_view(view),
                SetTexture(texture) => self.state.set_texture(texture),
                SetF32(uni, val) => self.state.set_f32(uni, val),
                SetVector2(uni, val) => self.state.set_vector2(uni, val),
                SetVector3(uni, val) => self.state.set_vector3(uni, val),
                SetMatrix4(uni, val) => self.state.set_matrix4(uni, val),
                EnableFrameBufferSRGB => self.state.enable_framebuffer_srgb(),
                DisableFrameBufferSRGB => self.state.disable_framebuffer_srgb(),
                EnableBlend => self.state.enable_blend(),
                DisableBlend => self.state.disable_blend(),
                EnableCullFace => self.state.enable_cull_face(),
                DisableCullFace => self.state.disable_cull_face(),
                CullFaceFront => self.state.cull_face_front(),
                CullFaceBack => self.state.cull_face_back(),
                CullFaceFrontAndBack => self.state.cull_face_front_and_back(),
                DrawTriangles(vertex_array, len) => self.state.draw_triangles(vertex_array, len),
                DrawTriangleStrip(vertex_array, len) => self.state.draw_triangle_strip(vertex_array, len),
                DrawLines(vertex_array, len) => self.state.draw_lines(vertex_array, len),
                DrawPoints(vertex_array, len) => self.state.draw_points(vertex_array, len),
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
}
