#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use turbine_scene3d::*;
use vecmath::{
    Matrix4,
    Vector3,
    Vector2,
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
    UV(UVBuffer),
    Nor(NormalBuffer),
}

enum Uni {
    F32(F32Uniform),
    Matrix4(Matrix4Uniform),
    Vector3(Vector3Uniform),
    Vector2(Vector2Uniform),
}

/// A render pipeline configuration.
#[derive(PartialEq, Eq)]
struct RpConf {
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
    blend: Option<wgpu::BlendState>,
}

impl RpConf {
    fn new(
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
        state: &State,
    ) -> Option<RpConf> {
        Some(RpConf {
            topology,
            polygon_mode,
            cull_mode: if state.cull_face {
                match (state.cull_face_front, state.cull_face_back) {
                    (false, false) => None,
                    (false, true) => Some(wgpu::Face::Back),
                    (true, false) => Some(wgpu::Face::Front),
                    (true, true) => return None,
                }
            } else {None},
            blend: if state.enable_blend {
                Some(state.blend_op)
            } else {None},
        })
    }
}

struct Prog {
    pub render_pipelines: Vec<(RpConf, wgpu::RenderPipeline)>,
    pub uniforms: Vec<Uni>,
    pub uniform_bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    pub uniform_bind_group: Option<wgpu::BindGroup>,
    pub vs: VertexShader,
    pub fs: FragmentShader,
}

impl Prog {
    fn new(vs: VertexShader, fs: FragmentShader) -> Prog {
        Prog {
            render_pipelines: vec![],
            uniforms: vec![],
            uniform_bind_group_layout_entries: vec![],
            uniform_bind_group: None,
            vs, fs,
        }
    }

    fn create_render_pipeline(
        &mut self,
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
        va: &Va,
        texture_layout: Option<&wgpu::BindGroupLayout>,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
        cull_mode: Option<wgpu::Face>,
        blend: Option<wgpu::BlendState>,
        device: &wgpu::Device,
    ) -> wgpu::RenderPipeline {
        use std::mem::size_of;
        use wgpu::VertexFormat::*;

        let mut vertex_attributes = vec![];
        for (attr, n) in &va.bufs {
            vertex_attributes.push(match n {
                Vert::Buf2(_) | Vert::UV(_) => wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: *attr,
                    format: Float32x2,
                },
                Vert::Buf3(_) | Vert::Nor(_) => wgpu::VertexAttribute {
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
                Vert::Buf2(_) | Vert::UV(_) => wgpu::VertexBufferLayout {
                    array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes[id..id + 1],
                },
                Vert::Buf3(_) | Vert::Nor(_) => wgpu::VertexBufferLayout {
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

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &self.uniform_bind_group_layout_entries,
        });

        let layouts = if let Some(texture_layout) = texture_layout {
            vec![&uniform_layout, texture_layout]
        } else {vec![&uniform_layout]};

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            }
        );

        utils::render_pipeline(
            &pipeline_layout,
            vs, fs,
            &vertex_buffer_layouts,
            &[Some(utils::color_target_state_replace(&surface_config, blend))],
            topology,
            polygon_mode,
            cull_mode,
            &device,
        )
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
    f32_uniforms: Vec<f32>,
    f32_uniform_buffers: Vec<wgpu::Buffer>,
    matrix4_uniforms: Vec<Matrix4<f32>>,
    matrix4_uniform_buffers: Vec<wgpu::Buffer>,
    vector3_uniforms: Vec<Vector3<f32>>,
    vector3_uniform_buffers: Vec<wgpu::Buffer>,
    vector2_uniforms: Vec<Vector2<f32>>,
    vector2_uniform_buffers: Vec<wgpu::Buffer>,
    vas: Vec<Va>,
    vertex_buffers: Vec<wgpu::Buffer>,
    textures: Vec<(wgpu::Texture, wgpu::BindGroupLayout, wgpu::BindGroup, u32, u32)>,
    depth_texture_view: wgpu::TextureView,
    encoder: Option<wgpu::CommandEncoder>,
    render_pass: Option<wgpu::RenderPass<'static>>,
    current_program: Option<Program>,
    current_texture: Option<Texture>,
    /// Whether to enable culling of faces.
    pub cull_face: bool,
    /// Whether to cull front faces.
    pub cull_face_front: bool,
    /// Whether to cull back faces.
    pub cull_face_back: bool,
    /// Whether blend is enabled.
    pub enable_blend: bool,
    /// The blend operation, if active.
    pub blend_op: wgpu::BlendState,
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
            f32_uniforms: vec![],
            f32_uniform_buffers: vec![],
            matrix4_uniforms: vec![],
            matrix4_uniform_buffers: vec![],
            vector3_uniforms: vec![],
            vector3_uniform_buffers: vec![],
            vector2_uniforms: vec![],
            vector2_uniform_buffers: vec![],
            vas: vec![],
            vertex_buffers: vec![],
            textures: vec![],
            depth_texture_view,
            cull_face: false,
            cull_face_back: false,
            cull_face_front: false,
            encoder: None,
            render_pass: None,
            surface_texture: None,
            current_program: None,
            current_texture: None,
            enable_blend: true,
            blend_op: wgpu::BlendState::REPLACE,
        }
    }

    fn draw(
        &mut self,
        va: VertexArray,
        n: usize,
        rp_conf: RpConf,
    ) {
        if let (Some(program), Some(render_pass)) =
            (self.current_program, self.render_pass.as_mut())
        {
            let prog = &mut self.programs[program.0];
            let mut ind: Option<usize> = prog.render_pipelines.iter().position(|n| n.0 == rp_conf);
            if ind.is_none() {
                ind = Some(prog.render_pipelines.len());
                let pipeline = prog.create_render_pipeline(
                    &self.shader_modules[prog.vs.0],
                    &self.shader_modules[prog.fs.0],
                    &self.surface_config,
                    &self.vas[va.0],
                    self.current_texture.map(|n| &self.textures[n.0].1),
                    rp_conf.topology,
                    rp_conf.polygon_mode,
                    rp_conf.cull_mode,
                    rp_conf.blend,
                    &self.device,
                );
                prog.render_pipelines.push((rp_conf, pipeline));
            }
            if prog.uniform_bind_group.is_none() {
                let mut entries: Vec<wgpu::BindGroupEntry> = vec![];
                for (binding, uni) in prog.uniforms.iter().enumerate() {
                    entries.push(match uni {
                        Uni::F32(s) => wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: self.f32_uniform_buffers[s.0].as_entire_binding(),
                        },
                        Uni::Matrix4(m) => wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: self.matrix4_uniform_buffers[m.0].as_entire_binding(),
                        },
                        Uni::Vector3(v) => wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: self.vector3_uniform_buffers[v.0].as_entire_binding(),
                        },
                        Uni::Vector2(v) => wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: self.vector2_uniform_buffers[v.0].as_entire_binding(),
                        },
                    });
                }
                let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniform_bind_group_layout"),
                    entries: &prog.uniform_bind_group_layout_entries,
                });
                prog.uniform_bind_group = Some(utils::uniform_bind_group(
                    &layout, &entries, &self.device));
            }
            if let (Some((_, render_pipeline)), Some(uniform_bind_group)) =
                (prog.render_pipelines.get(ind.unwrap()), prog.uniform_bind_group.as_ref())
            {
                render_pass.set_pipeline(render_pipeline);
                render_pass.set_bind_group(0, uniform_bind_group, &[]);
                if let Some(tex) = self.current_texture {
                    render_pass.set_bind_group(1, &self.textures[tex.0].2, &[]);
                }
                for (attr, buf) in &self.vas[va.0].bufs {
                    let ind = match buf {
                        Vert::Buf2(VertexBuffer2(x, _)) |
                        Vert::Buf3(VertexBuffer3(x, _)) |
                        Vert::Col(ColorBuffer(x, _)) |
                        Vert::UV(UVBuffer(x, _)) |
                        Vert::Nor(NormalBuffer(x, _)) => *x,
                    };
                    render_pass.set_vertex_buffer(*attr, self.vertex_buffers[ind].slice(..));
                }
                render_pass.draw(0..n as u32, 0..1);
            }
            self.queue.submit([]);
        }
    }
}

impl Backend for State {
    type ImageError = image::ImageError;

    fn set_texture(&mut self, tex: turbine_scene3d::Texture) {
        self.current_texture = Some(tex);
    }
    fn draw_points(&mut self, va: turbine_scene3d::VertexArray, n: usize) {
        if let Some(rp_conf) = RpConf::new(
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Point,
            self,
        ) {
            self.draw(va, n, rp_conf);
        }
    }
    fn draw_lines(&mut self, va: turbine_scene3d::VertexArray, n: usize) {
        if let Some(rp_conf) = RpConf::new(
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Line,
            self,
        ) {
            self.draw(va, n, rp_conf);
        }
    }
    fn draw_triangle_strip(&mut self, va: VertexArray, n: usize) {
        if let Some(rp_conf) = RpConf::new(
            wgpu::PrimitiveTopology::TriangleStrip,
            wgpu::PolygonMode::Fill,
            self,
        ) {
            self.draw(va, n, rp_conf);
        }
    }
    fn draw_triangles(&mut self, va: VertexArray, n: usize) {
        if let Some(rp_conf) = RpConf::new(
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
            self,
        ) {
            self.draw(va, n, rp_conf);
        }
    }
    fn enable_framebuffer_srgb(&mut self) {
        // This is not supported in the WGPU backend.
    }
    fn disable_framebuffer_srgb(&mut self) {
        // This is not supported in the WGPU backend.
    }
    fn enable_blend(&mut self) {self.enable_blend = true}
    fn disable_blend(&mut self) {self.enable_blend = false}
    fn enable_cull_face(&mut self) {self.cull_face = true}
    fn disable_cull_face(&mut self) {self.cull_face = false}
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
    fn set_f32(&mut self, uni: F32Uniform, s: f32) {
        let rendering = self.surface_texture.is_some();
        self.end_render_pass();

        self.f32_uniforms[uni.0] = s;
        self.queue.write_buffer(
            &self.f32_uniform_buffers[uni.0],
            0,
            bytemuck::cast_slice(&[s]),
        );
        self.queue.submit([]);

        if rendering {self.begin_render_pass()};
    }
    fn set_vector2(&mut self, uni: Vector2Uniform, v: [f32; 2]) {
        let rendering = self.surface_texture.is_some();
        self.end_render_pass();

        self.vector2_uniforms[uni.0] = v;
        self.queue.write_buffer(
            &self.vector2_uniform_buffers[uni.0],
            0,
            bytemuck::cast_slice(&[v]),
        );
        self.queue.submit([]);

        if rendering {self.begin_render_pass()};
    }
    fn set_vector3(&mut self, uni: Vector3Uniform, v: [f32; 3]) {
        let rendering = self.surface_texture.is_some();
        self.end_render_pass();

        self.vector3_uniforms[uni.0] = v;
        self.queue.write_buffer(
            &self.vector3_uniform_buffers[uni.0],
            0,
            bytemuck::cast_slice(&[v]),
        );
        self.queue.submit([]);

        if rendering {self.begin_render_pass()};
    }
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
    fn matrix4_uniform(&mut self, program: Program, _: &str) -> Result<Matrix4Uniform, String> {
        let id = Matrix4Uniform(self.matrix4_uniforms.len());

        let binding = self.programs[program.0].uniforms.len();

        let uniform = vecmath::mat4_id();
        let uniform_buffer = utils::matrix4_uniform_buffer(uniform, &self.device);
        self.programs[program.0].uniform_bind_group_layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: binding as u32,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }
        );

        self.matrix4_uniforms.push(uniform);
        self.matrix4_uniform_buffers.push(uniform_buffer);

        self.programs[program.0].uniforms.push(Uni::Matrix4(id));
        Ok(id)
    }
    fn program_from_vertex_fragment(&mut self, vs: VertexShader, fs: FragmentShader) -> turbine_scene3d::Program {
        let id = self.programs.len();
        self.programs.push(Prog::new(vs, fs));
        Program(id)
    }
    fn f32_uniform(&mut self, program: Program, _: &str) -> Result<F32Uniform, String> {
        let id = F32Uniform(self.f32_uniforms.len());

        let binding = self.programs[program.0].uniforms.len();

        let uniform = 0.0;
        let uniform_buffer = utils::f32_uniform_buffer(uniform, &self.device);
        self.programs[program.0].uniform_bind_group_layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: binding as u32,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }
        );

        self.f32_uniforms.push(uniform);
        self.f32_uniform_buffers.push(uniform_buffer);

        self.programs[program.0].uniforms.push(Uni::F32(id));
        Ok(id)
    }
    fn vector2_uniform(&mut self, program: Program, _: &str) -> Result<Vector2Uniform, String> {
        let id = Vector2Uniform(self.vector2_uniforms.len());

        let binding = self.programs[program.0].uniforms.len();

        let uniform = [0.0; 2];
        let uniform_buffer = utils::vector2_uniform_buffer(uniform, &self.device);
        self.programs[program.0].uniform_bind_group_layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: binding as u32,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }
        );

        self.vector2_uniforms.push(uniform);
        self.vector2_uniform_buffers.push(uniform_buffer);

        self.programs[program.0].uniforms.push(Uni::Vector2(id));
        Ok(id)
    }
    fn vector3_uniform(&mut self, program: Program, _: &str) -> Result<Vector3Uniform, String> {
        let id = Vector3Uniform(self.vector3_uniforms.len());

        let binding = self.programs[program.0].uniforms.len();

        let uniform = [0.0; 3];
        let uniform_buffer = utils::vector3_uniform_buffer(uniform, &self.device);
        self.programs[program.0].uniform_bind_group_layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: binding as u32,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }
        );

        self.vector3_uniforms.push(uniform);
        self.vector3_uniform_buffers.push(uniform_buffer);

        self.programs[program.0].uniforms.push(Uni::Vector3(id));
        Ok(id)
    }
    fn load_texture<P>(&mut self, path: P) -> Result<Texture, <Self as turbine_scene3d::Backend>::ImageError> where P: AsRef<Path> {
        let image = match image::open(path)? {
            image::DynamicImage::ImageRgba8(img) => img,
            x => x.to_rgba8()
        };
        let (width, height) = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Diffuse Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture View"),
            ..Default::default()
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
            ..Default::default()
        });

        let bind_group_layout = utils::texture_bind_group_layout(&self.device);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let id = self.textures.len();
        self.textures.push((
            texture,
            bind_group_layout,
            bind_group,
            width,
            height,
        ));
        Ok(Texture(id))
    }
    fn normal_buffer(&mut self, va: VertexArray, attr: u32, data: &[f32]) -> NormalBuffer {
        let id = NormalBuffer(self.vertex_buffers.len(), data.len() / 4);
        self.vertex_buffers.push(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("NormalBuffer"),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ));
        let va = &mut self.vas[va.0];
        va.bufs.push((attr, Vert::Nor(id)));
        id
    }
    fn uv_buffer(&mut self, va: VertexArray, attr: u32, data: &[f32]) -> UVBuffer {
        let id = UVBuffer(self.vertex_buffers.len(), data.len() / 4);
        self.vertex_buffers.push(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("UVBuffer"),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ));
        let va = &mut self.vas[va.0];
        va.bufs.push((attr, Vert::UV(id)));
        id
    }
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
