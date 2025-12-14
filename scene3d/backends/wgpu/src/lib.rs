#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use turbine_scene3d::*;
use vecmath::{
    Matrix4,
};
use wgpu::util::DeviceExt;

use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;

pub mod utils;

struct Va {
    pub bufs: Vec<(u32, Vert)>,
}

impl Va {
    fn new() -> Va {
        Va {
            bufs: vec![]
        }
    }
}

enum Vert {
    Buf2(VertexBuffer2),
    Buf3(VertexBuffer3),
    Col(ColorBuffer),
}

enum Uni {
    Matrix4(Matrix4Uniform),
}

struct Prog {
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub uniforms: Vec<Uni>,
    pub uniform_bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub vs: VertexShader,
    pub fs: FragmentShader,
}

impl Prog {
    fn new(vs: VertexShader, fs: FragmentShader) -> Prog {
        Prog {
            render_pipeline: None,
            uniforms: vec![],
            uniform_bind_group_layouts: vec![],
            vs, fs,
        }
    }

    fn create_render_pipeline(
        &mut self,
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
        va: &Va,
        device: &wgpu::Device,
    ) {
        use std::mem::size_of;
        use wgpu::VertexFormat::*;

        let mut vertex_attributes = vec![];
        for (attr, n) in &va.bufs {
            vertex_attributes.push(match n {
                Vert::Buf2(_) => wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: *attr,
                    format: Float32x2,
                },
                Vert::Buf3(_) => wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: *attr,
                    format: Float32x3,
                },
                Vert::Col(_) => wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: *attr,
                    format: Float32x4,
                },
            });
        }
        let mut vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout> = vec![];
        for (id, (_, n)) in va.bufs.iter().enumerate() {
            vertex_buffer_layouts.push(match n {
                Vert::Buf2(_) => wgpu::VertexBufferLayout {
                    array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes[id..id + 1],
                },
                Vert::Buf3(_) => wgpu::VertexBufferLayout {
                    array_stride: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes[id..id + 1],
                },
                Vert::Col(_) => wgpu::VertexBufferLayout {
                    array_stride: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes[id..id + 1],
                },
            });
        }

        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> =
            self.uniform_bind_group_layouts.iter().collect();
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Color3D Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            }
        );

        self.render_pipeline = Some(utils::render_pipeline(
            &pipeline_layout,
            vs, fs,
            &vertex_buffer_layouts,
            &[Some(utils::color_target_state_replace(&surface_config))],
            &device,
        ));
    }
}

/// Stores the state of the WGPU backend.
pub struct State {
    /// The surface texture used for rendering.
    ///
    /// This is set to `None` when not rendering.
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    surface_config: wgpu::SurfaceConfiguration,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    shader_modules: Vec<wgpu::ShaderModule>,
    shader_cache: HashMap<String, usize>,
    programs: Vec<Prog>,
    matrix4_uniforms: Vec<Matrix4<f32>>,
    matrix4_uniform_buffers: Vec<wgpu::Buffer>,
    matrix4_uniform_bind_groups: Vec<wgpu::BindGroup>,
    vas: Vec<Va>,
    vertex_buffers: Vec<wgpu::Buffer>,
    depth_texture_view: wgpu::TextureView,
    cull_face: bool,
    cull_face_front: bool,
    cull_face_back: bool,
    encoder: Option<wgpu::CommandEncoder>,
    render_pass: Option<wgpu::RenderPass<'static>>,
    current_program: Option<Program>,
}

impl State {
    /// Creates a new state.
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_config: wgpu::SurfaceConfiguration,
        depth_texture_view: wgpu::TextureView,
    ) -> State {
        State {
            surface_config,
            device,
            queue,
            shader_modules: vec![],
            shader_cache: HashMap::new(),
            programs: vec![],
            matrix4_uniforms: vec![],
            matrix4_uniform_buffers: vec![],
            matrix4_uniform_bind_groups: vec![],
            vas: vec![],
            vertex_buffers: vec![],
            depth_texture_view,
            cull_face: false,
            cull_face_back: false,
            cull_face_front: false,
            encoder: None,
            render_pass: None,
            surface_texture: None,
            current_program: None,
        }
    }
}

impl Backend for State {
    type ImageError = image::ImageError;

    fn set_texture(&mut self, _: turbine_scene3d::Texture) { todo!() }
    fn draw_points(&mut self, _: turbine_scene3d::VertexArray, _: usize) { todo!() }
    fn draw_lines(&mut self, _: turbine_scene3d::VertexArray, _: usize) { todo!() }
    fn draw_triangle_strip(&mut self, _: turbine_scene3d::VertexArray, _: usize) { todo!() }
    fn draw_triangles(&mut self, va: turbine_scene3d::VertexArray, n: usize) {
        if let (Some(program), Some(render_pass)) = (self.current_program, self.render_pass.as_mut()) {
            let prog = &mut self.programs[program.0];
            if prog.render_pipeline.is_none() {
                prog.create_render_pipeline(
                    &self.shader_modules[prog.vs.0],
                    &self.shader_modules[prog.fs.0],
                    &self.surface_config,
                    &self.vas[va.0],
                    &self.device,
                );
            }
            if let Some(render_pipeline) = prog.render_pipeline.as_ref() {
                render_pass.set_pipeline(render_pipeline);
                for uni in prog.uniforms.iter() {
                    match uni {
                        Uni::Matrix4(m) => {
                            render_pass.set_bind_group(0,
                                &self.matrix4_uniform_bind_groups[m.0], &[]);
                        }
                    }
                }
                for (attr, buf) in &self.vas[va.0].bufs {
                    let ind = match buf {
                        Vert::Buf2(VertexBuffer2(x, _)) |
                        Vert::Buf3(VertexBuffer3(x, _)) |
                        Vert::Col(ColorBuffer(x, _)) => *x,
                    };
                    render_pass.set_vertex_buffer(*attr, self.vertex_buffers[ind].slice(..));
                }
                render_pass.draw(0..n as u32, 0..1);
            }
            self.queue.submit([]);
        }
    }
    fn enable_framebuffer_srgb(&mut self) { todo!() }
    fn disable_framebuffer_srgb(&mut self) { todo!() }
    fn enable_blend(&mut self) { todo!() }
    fn disable_blend(&mut self) { todo!() }
    fn enable_cull_face(&mut self) {self.cull_face = true}
    fn disable_cull_face(&mut self) {
        self.cull_face = false;
        self.cull_face_front = false;
        self.cull_face_back = false;
    }
    fn cull_face_front(&mut self) {self.cull_face_front = true}
    fn cull_face_back(&mut self) {self.cull_face_back = true}
    fn cull_face_front_and_back(&mut self) {
        self.cull_face_front = true;
        self.cull_face_back = true;
    }
    fn clear(&mut self, color: [f32; 4], _settings: &turbine_scene3d::SceneSettings) {
        let surface_texture = self.surface_texture.as_ref().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = &self.depth_texture_view;

        let clear_color = wgpu::Color {
            r: color[0] as f64,
            g: color[1] as f64,
            b: color[2] as f64,
            a: color[3] as f64,
        };
        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let rpass_color_attachment = wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            };
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(rpass_color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
    }
    fn set_f32(&mut self, _: turbine_scene3d::F32Uniform, _: f32) { todo!() }
    fn set_vector2(&mut self, _: turbine_scene3d::Vector2Uniform, _: [f32; 2]) { todo!() }
    fn set_vector3(&mut self, _: turbine_scene3d::Vector3Uniform, _: [f32; 3]) { todo!() }
    fn set_matrix4(&mut self, uni: turbine_scene3d::Matrix4Uniform, mat: [[f32; 4]; 4]) {
        let rendering = self.surface_texture.is_some();
        self.end_render_pass();

        self.matrix4_uniforms[uni.0] = mat;
        self.queue.write_buffer(
            &self.matrix4_uniform_buffers[uni.0],
            0,
            bytemuck::cast_slice(&[mat]),
        );
        self.queue.submit([]);

        if rendering {self.begin_render_pass()};
    }
    fn use_program(&mut self, program: turbine_scene3d::Program) {
        self.current_program = Some(program);
    }
    fn vertex_shader(&mut self, src: &str) -> std::result::Result<turbine_scene3d::VertexShader, String> {
        if let Some(id) = self.shader_cache.get(src) {
            Ok(VertexShader(*id))
        } else {
            let id = self.shader_modules.len();
            self.shader_modules.push({
                use std::borrow::Cow;
                use wgpu::{Label, ShaderModuleDescriptor, ShaderSource};
                self.device.create_shader_module(
                    ShaderModuleDescriptor {
                        label: Label::None,
                        source: ShaderSource::Wgsl(Cow::Borrowed(src))
                    }
                )
            });
            self.shader_cache.insert(src.to_string(), id);
            Ok(VertexShader(id))
        }
    }
    fn fragment_shader(&mut self, src: &str) -> std::result::Result<turbine_scene3d::FragmentShader, String> {
        if let Some(id) = self.shader_cache.get(src) {
            Ok(FragmentShader(*id))
        } else {
            let id = self.shader_modules.len();
            self.shader_modules.push({
                use std::borrow::Cow;
                use wgpu::{Label, ShaderModuleDescriptor, ShaderSource};
                self.device.create_shader_module(
                    ShaderModuleDescriptor {
                        label: Label::None,
                        source: ShaderSource::Wgsl(Cow::Borrowed(src))
                    }
                )
            });
            self.shader_cache.insert(src.to_string(), id);
            Ok(FragmentShader(id))
        }
    }
    fn vertex_buffer2(&mut self, va: turbine_scene3d::VertexArray, attr: u32, data: &[f32]) -> VertexBuffer2 {
        let id = VertexBuffer2(0, data.len() / 2);
        let va = &mut self.vas[va.0];
        va.bufs.push((attr, Vert::Buf2(id)));
        id
    }
    fn vertex_buffer3(&mut self, va: turbine_scene3d::VertexArray, attr: u32, data: &[f32]) -> VertexBuffer3 {
        let id = VertexBuffer3(self.vertex_buffers.len(), data.len() / 3);
        self.vertex_buffers.push(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("VertexBuffer3"),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ));
        let va = &mut self.vas[va.0];
        va.bufs.push((attr, Vert::Buf3(id)));
        id
    }
    fn color_buffer(&mut self, va: turbine_scene3d::VertexArray, attr: u32, data: &[f32]) -> ColorBuffer {
        let id = ColorBuffer(self.vertex_buffers.len(), data.len() / 4);
        self.vertex_buffers.push(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("ColorBuffer"),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ));
        let va = &mut self.vas[va.0];
        va.bufs.push((attr, Vert::Col(id)));
        id
    }
    fn vertex_array(&mut self) -> turbine_scene3d::VertexArray {
        let id = self.vas.len();
        self.vas.push(Va::new());
        VertexArray(id)
    }
    fn matrix4_uniform(&mut self, program: turbine_scene3d::Program, _: &str) -> std::result::Result<Matrix4Uniform, String> {
        let id = Matrix4Uniform(self.matrix4_uniforms.len());

        let binding = self.programs[program.0].uniforms.len();

        let uniform = vecmath::mat4_id();
        let uniform_buffer = utils::matrix4_uniform_buffer(uniform, &self.device);
        let uniform_bind_group_layout = utils::uniform_bind_group_layout(binding as u32, &self.device);
        self.programs[program.0].uniform_bind_group_layouts.push(uniform_bind_group_layout.clone());

        let uniform_bind_group = utils::uniform_bind_group(
            &uniform_bind_group_layout,
            &uniform_buffer,
            &self.device,
        );

        self.matrix4_uniforms.push(uniform);
        self.matrix4_uniform_buffers.push(uniform_buffer);
        self.matrix4_uniform_bind_groups.push(uniform_bind_group);

        self.programs[program.0].uniforms.push(Uni::Matrix4(id));
        Ok(id)
    }
    fn program_from_vertex_fragment(&mut self, vs: VertexShader, fs: FragmentShader) -> turbine_scene3d::Program {
        let id = self.programs.len();
        self.programs.push(Prog::new(vs, fs));
        Program(id)
    }
    fn f32_uniform(&mut self, _: turbine_scene3d::Program, _: &str) -> std::result::Result<F32Uniform, String> { todo!() }
    fn vector2_uniform(&mut self, _: turbine_scene3d::Program, _: &str) -> std::result::Result<Vector2Uniform, String> { todo!() }
    fn vector3_uniform(&mut self, _: Program, _: &str) -> std::result::Result<Vector3Uniform, String> { todo!() }
    fn load_texture<P>(&mut self, _: P) -> std::result::Result<turbine_scene3d::Texture, <Self as turbine_scene3d::Backend>::ImageError> where P: AsRef<Path> { todo!() }
    fn normal_buffer(&mut self, _: turbine_scene3d::VertexArray, _: u32, _: &[f32]) -> NormalBuffer { todo!() }
    fn uv_buffer(&mut self, _: VertexArray, _: u32, _: &[f32]) -> UVBuffer { todo!() }
}

impl State {
    /// Start a new render pass.
    pub fn begin_render_pass(&mut self) {
        let surface_texture = self.surface_texture.as_ref().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = &self.depth_texture_view;

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: None,
            }
        );

        let rpass_color_attachment = wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            };
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(rpass_color_attachment)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        }).forget_lifetime();

        self.encoder = Some(encoder);
        self.render_pass = Some(render_pass);
    }

    /// End current render pass.
    pub fn end_render_pass(&mut self) {
        self.render_pass = None;
        if let Some(encoder) = std::mem::replace(&mut self.encoder, None) {
            self.queue.submit(Some(encoder.finish()));
        }
    }
}
