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

extern crate piston;
extern crate vecmath;
extern crate wavefront_obj;
extern crate image;

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
mod rendy_backend;
#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
mod gl_backend;

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
pub use rendy_backend::Scene;
#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
pub use gl_backend::Scene;

use std::path::Path;
use std::io;

use vecmath::*;

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
    /// Set f32 uniform.
    SetF32(F32Uniform, f32),
    /// Set 2D vector uniform.
    SetVector2(Vector2Uniform, Vector2<f32>),
    /// Set 3D vector uniform.
    SetVector3(Vector3Uniform, Vector3<f32>),
    /// Set matrx uniform.
    SetMatrix4(Matrix4Uniform, Matrix4<f32>),
    /// Enable framebuffer sRGB.
    EnableFrameBufferSRGB,
    /// Disable framebuffer sRGB.
    DisableFrameBufferSRGB,
    /// Enable blend.
    EnableBlend,
    /// Disable blend.
    DisableBlend,
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
/// References a 3D vertex buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexBuffer3(usize, usize);
/// References a 2D vertex buffer object.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexBuffer2(usize, usize);
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

impl VertexBuffer3 {
    /// Length of vertex buffer.
    pub fn len(&self) -> usize {self.1}
}

impl VertexBuffer2 {
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

/// Stores scene settings.
#[derive(Clone)]
pub struct SceneSettings {
    clear_depth_buffer: bool,
    clear_enable_depth_test: bool,
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

impl Default for SceneSettings {
    fn default() -> Self {SceneSettings::new()}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
