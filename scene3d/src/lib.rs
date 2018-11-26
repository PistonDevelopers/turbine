//! # Turbine-Scene3D
//!
//! Scene rendering for the Turbine game engine.
//!
//! <video width="320" height="240" controls>
//!  <source src="https://i.imgur.com/M0frz9B.mp4" type="video/mp4">
//! Your browser does not support the video tag.
//! </video>
//!
//! ### Design
//!
//! - Scene object stores all resources used for rendering
//! - Frame graph stores command lists
//!
//! This design allows flexible programming of scenes, without the need for
//! a tree structure to store nodes for scene data.
//! The frame graph can be used to debug the scene.

#![deny(missing_docs)]

extern crate gl;
extern crate piston;
extern crate vecmath;
extern crate opengl_graphics;
extern crate wavefront_obj;
extern crate image;

use std::path::Path;
use std::io;

use vecmath::*;
use opengl_graphics::shader_utils::{
    compile_shader,
    uniform_location
};

/// Stores a scene command.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Command {
    /// Use program.
    UseProgram(Program),
    /// Set model-view-projection transform.
    SetModelViewProjection(Matrix4Uniform),
    /// Set model transform.
    SetModel(Matrix4Uniform),
    /// Set view transform.
    SetView(Matrix4Uniform),
    /// Set texture.
    SetTexture(Texture),
    /// Enable cull face.
    EnableCullFace,
    /// Disable cull face.
    DisableCullFace,
    /// Cull front face.
    CullFaceFront,
    /// Cull back face.
    CullFaceBack,
    /// Cull both front and back face.
    CullFaceFrontAndBack,
    /// Draw triangles.
    DrawTriangles(VertexArray, usize),
    /// Draw triangle strip.
    DrawTriangleStrip(VertexArray, usize),
    /// Draw lines.
    DrawLines(VertexArray, usize),
    /// Draw points.
    DrawPoints(VertexArray, usize),
    /// Translate model.
    Translate(Vector3<f32>),
    /// Translate model in global coordinates.
    TranslateGlobal(Vector3<f32>),
    /// Scale model.
    Scale(Vector3<f32>),
    /// Rotate model around x axis with degrees.
    RotateXDeg(f32),
    /// Rotate model around y axis with degrees.
    RotateYDeg(f32),
    /// Rotate model around z axis with degrees.
    RotateZDeg(f32),
    /// Rotate model around axis with degrees.
    RotateAxisDeg(Vector3<f32>, f32),
    /// Push model transform to transform stack.
    PushTransform,
    /// Pop model transform from transform stack.
    PopTransform,
    /// Draw a command list.
    Draw(CommandList),
}

/// Stores how stuff is rendered in a single frame.
#[derive(Debug)]
pub struct FrameGraph {
    command_lists: Vec<Vec<Command>>,
}

impl FrameGraph {
    /// Creates a new frame graph.
    pub fn new() -> FrameGraph {
        FrameGraph {
            command_lists: vec![]
        }
    }

    /// Create command list.
    pub fn command_list(&mut self, commands: Vec<Command>) -> CommandList {
        let id = self.command_lists.len();
        self.command_lists.push(commands);
        CommandList(id)
    }
}

/// References a vertex shader.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexShader(usize);
/// References a fragment shader.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FragmentShader(usize);
/// References a program.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program(usize);
/// References 4D matrix uniform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Matrix4Uniform(usize);
/// References a 2D vector uniform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vector2Uniform(usize);
/// References a 3D vector uniform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vector3Uniform(usize);
/// References a f32 uniform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct F32Uniform(usize);
/// References a vertex array object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexArray(usize);
/// References a color buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColorBuffer(usize, usize);
/// References a vertex buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexBuffer(usize, usize);
/// References an UV buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UVBuffer(usize, usize);
/// References a normal buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NormalBuffer(usize, usize);
/// References a command list object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommandList(usize);
/// References a texture object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Texture(usize);

impl ColorBuffer {
    /// Length of color buffer.
    pub fn len(&self) -> usize {self.1}
}

impl VertexBuffer {
    /// Length of vertex buffer.
    pub fn len(&self) -> usize {self.1}
}

/// Stores OBJ mesh data.
pub struct ObjMesh {
    /// Stores vertex coordinates.
    pub vertices: Vec<f32>,
    /// Stores texture coordinates.
    pub uvs: Vec<f32>,
    /// Stores normal coordinates.
    pub normals: Vec<f32>,
}

impl ObjMesh {
    /// Load OBJ file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ObjMesh, io::Error> {
        use std::fs::File;
        use std::io::Read;

        let mut obj_file = File::open(path)?;
        let mut data = String::new();
        obj_file.read_to_string(&mut data)?;
        let obj_set = wavefront_obj::obj::parse(data).unwrap();
        let obj = &obj_set.objects[0];
        let temp_vertices = {
            let mut res = vec![];
            for v in &obj.vertices {
                res.push(v.x as f32);
                res.push(v.y as f32);
                res.push(v.z as f32);
            }
            res
        };
        let temp_uvs = {
            let mut res = vec![];
            for uv in &obj.tex_vertices {
                res.push(uv.u as f32);
                res.push(1.0 - uv.v as f32);
            }
            res
        };
        let temp_normals = {
            let mut res = vec![];
            for normal in &obj.normals {
                res.push(normal.x as gl::types::GLfloat);
                res.push(normal.y as gl::types::GLfloat);
                res.push(normal.z as gl::types::GLfloat);
            }
            res
        };
        let mut vertices = vec![];
        let mut uvs = vec![];
        let mut normals = vec![];
        for geom in &obj.geometry {
            for shape in &geom.shapes {
                use wavefront_obj::obj::Primitive;

                if let Primitive::Triangle(
                    (a_v, Some(a_uv), Some(a_n)),
                    (b_v, Some(b_uv), Some(b_n)),
                    (c_v, Some(c_uv), Some(c_n))
                ) = shape.primitive {
                    vertices.push(temp_vertices[a_v * 3 + 0]);
                    vertices.push(temp_vertices[a_v * 3 + 1]);
                    vertices.push(temp_vertices[a_v * 3 + 2]);

                    vertices.push(temp_vertices[b_v * 3 + 0]);
                    vertices.push(temp_vertices[b_v * 3 + 1]);
                    vertices.push(temp_vertices[b_v * 3 + 2]);

                    vertices.push(temp_vertices[c_v * 3 + 0]);
                    vertices.push(temp_vertices[c_v * 3 + 1]);
                    vertices.push(temp_vertices[c_v * 3 + 2]);

                    uvs.push(temp_uvs[a_uv * 2 + 0]);
                    uvs.push(temp_uvs[a_uv * 2 + 1]);

                    uvs.push(temp_uvs[b_uv * 2 + 0]);
                    uvs.push(temp_uvs[b_uv * 2 + 1]);

                    uvs.push(temp_uvs[c_uv * 2 + 0]);
                    uvs.push(temp_uvs[c_uv * 2 + 1]);

                    normals.push(temp_normals[a_n * 3 + 0]);
                    normals.push(temp_normals[a_n * 3 + 1]);
                    normals.push(temp_normals[a_n * 3 + 2]);

                    normals.push(temp_normals[b_n * 3 + 0]);
                    normals.push(temp_normals[b_n * 3 + 1]);
                    normals.push(temp_normals[b_n * 3 + 2]);

                    normals.push(temp_normals[c_n * 3 + 0]);
                    normals.push(temp_normals[c_n * 3 + 1]);
                    normals.push(temp_normals[c_n * 3 + 2]);
                }
            }
        }
        Ok(ObjMesh {
            vertices,
            uvs,
            normals
        })
    }
}

/// Stores scene data.
pub struct Scene {
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
    pub fn new() -> Scene {
        let mat_id = mat4_id();
        Scene {
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

    /// Load texture from path.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, image::ImageError> {
        use std::mem::transmute;

        let image = match image::open(path)? {
            image::DynamicImage::ImageRgba8(img) => img,
            x => x.to_rgba()
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
    pub fn vertex_buffer_2d(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer {
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
            VertexBuffer(id, data.len() / 2)
        }
    }

    /// Create vertex buffer for 3D coordinates.
    pub fn vertex_buffer(
        &mut self,
        vertex_array: VertexArray,
        attribute: u32,
        data: &[f32]
    ) -> VertexBuffer {
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
            VertexBuffer(id, data.len() / 3)
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
        use vecmath::col_mat4_mul as mul;
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
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
