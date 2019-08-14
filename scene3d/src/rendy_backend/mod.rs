#![allow(dead_code)]
#![cfg_attr(
    not(any(feature = "dx12", feature = "metal", feature = "vulkan")),
    allow(unused)
)]

extern crate rendy;
extern crate spirv_reflect;
extern crate serde;
extern crate serde_json;
extern crate winit_window;

use std::{
    path::Path, 
    io::BufReader,
    fs::File,
    option::Option,
    mem::size_of
};
use self::rendy::{
    command::{RenderPassEncoder, QueueId},
    factory::{Config, Factory, ImageState},
    graph::{render, Graph, GraphContext, NodeBuffer, NodeImage, GraphBuilder, BufferAccess, ImageAccess, present::PresentNode},
    hal::{self, PhysicalDevice, Device},
    resource::{DescriptorSet, Buffer, Escape, DescriptorSetLayout, Handle, BufferInfo},
    wsi::winit,
    mesh::{Position, Color, TexCoord, Normal, VertexFormat, Mesh},
    memory::{Dynamic},
    shader,
    texture
};
use vecmath::*;
use crate::*;
use rendy_backend::rendy::{
    graph::{render::*},
    mesh::{AsVertex}
};
use self::spirv_reflect::{
    ShaderModule, 
    types::ReflectDescriptorBinding,
    types::ReflectBlockVariable
};
use self::winit_window::WinitWindow;
use piston::window::WindowSettings;

/// Backends selected by features
#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;

// Indicates how many UBOs can exist in a frame, necessary for dynamic uniforms during a draw
const UBOS_PER_FRAME_SIZE: u64 = 4;

// Enums to index the different types of pipeline states the framegraph can swap through
#[derive(Debug, Copy, Clone)]
pub enum PipelineDrawMode {
    Fill  = 0,
    Line  = 1,
    Point = 2,
    Size  = 3,
}

#[derive(Debug, Copy, Clone)]
pub enum PipelineCullMode {
    Off       = 0,
    Front     = 1,
    Back      = 2,
    FrontBack = 3,
    Size      = 4,
}

#[allow(type_alias_bounds)]
type PipelineModes<B: hal::Backend> = [[B::GraphicsPipeline; PipelineCullMode::Size as usize]; PipelineDrawMode::Size as usize];

// Rendy doesn't implement this :)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PosColorTexNorm {
    /// Position of the vertex in 3D space.
    pub position: Position,
    /// Tangent vector of the vertex.
    pub color: Color,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: TexCoord,
    /// Normal vector of the vertex.
    pub normal: Normal,
}

impl AsVertex for PosColorTexNorm {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Position::vertex()),
            (Color::vertex()),
            (TexCoord::vertex()),
            (Normal::vertex()),
        ))
    }
}

/// Vertex Array Data
#[derive(Debug)]
pub struct VertexArrayData<B>
where
    B: hal::Backend
{
    pub buffer: Option<Mesh<B>>,
    pub vertices: Option<Vec<PosColorTexNorm>>,
    pub uploaded: bool,
}

impl<B> VertexArrayData<B> 
where
    B: hal::Backend
{
    pub fn new() -> VertexArrayData<B> {
        VertexArrayData {
            buffer: None,
            vertices: None,
            uploaded: false
        }
    }

    fn reset(&mut self, size: usize) {
        let rng = 0..size;
        self.vertices = Some(
            rng
                .map(|_|
                    PosColorTexNorm {
                        position: Position([0.0, 0.0, 0.0]),
                        color: Color([0.0, 0.0, 0.0, 0.0]),
                        tex_coord: TexCoord([0.0, 0.0]),
                        normal: Normal([0.0, 0.0, 0.0])
                    }
                )
                .collect()
        )
    }

    pub fn add_positions(&mut self, data: &[f32]) {
        if self.vertices.is_none() {
            self.reset(data.len()/3)
        }
        let mut pos_chunks = data.chunks(3);
        for vertex in self.vertices.as_mut().unwrap() {
            let pos = pos_chunks.next().unwrap();
            vertex.position = Position([pos[0], pos[1], pos[2]]);
        }
    }

    pub fn add_colors(&mut self, data: &[f32]) {
        if self.vertices.is_none() {
            self.reset(data.len()/4)
        }
        let mut color_chunks = data.chunks(4);
        for vertex in self.vertices.as_mut().unwrap() {
            let color = color_chunks.next().unwrap();
            vertex.color = Color([color[0], color[1], color[2], color[3]]);
        }
    }

    pub fn add_tex_coords(&mut self, data: &[f32]) {
        if self.vertices.is_none() {
            self.reset(data.len()/2)
        }
        let mut tex_chunks = data.chunks(2);
        for vertex in self.vertices.as_mut().unwrap() {
            let tex = tex_chunks.next().unwrap();
            // DX12, Vulkan, and Metal all invert the U coordinate in comparison to OpenGL
            // Handy charts: https://github.com/gfx-rs/gfx/tree/master/src/backend
            vertex.tex_coord = TexCoord([tex[0], tex[1]]);
        }
    }

    pub fn add_normals(&mut self, data: &[f32]) {
        if self.vertices.is_none() {
            self.reset(data.len()/3)
        }
        let mut norm_chunks = data.chunks(3);
        for vertex in self.vertices.as_mut().unwrap() {
            let norm = norm_chunks.next().unwrap();
            vertex.normal = Normal([norm[0], norm[1], norm[2]]);
        }
    }

    pub fn upload_check(&mut self, factory: &Factory<B>, queue: QueueId) {
        if self.vertices.is_none() || self.uploaded {
            return
        }
        else {
            self.buffer = Some(
                Mesh::<B>::builder()
                    .with_vertices(&self.vertices.as_ref().unwrap()[..])
                    .build(queue, &factory)
                    .unwrap()
            );
            self.uploaded = true;
        }
    }
}

/// Shader Data
#[derive(Debug)]
pub struct ShaderData {
    pub shader: rendy::shader::ShaderSetBuilder,
    pub reflection: shader::SpirvReflection,
    pub uniform_map: Vec<ReflectDescriptorBinding>
}

fn get_descriptors(ss: &shader::SpirvShader) -> Vec<ReflectDescriptorBinding> {
    // HACK: taking advantage of serde to get spirv data for reflection (yikes)
    let s = serde_json::to_value(&ss).unwrap();
    let s_spirv: Vec<u8> = s["spirv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_u64().unwrap() as u8)
        .collect();

    ShaderModule::load_u8_data(s_spirv.as_slice())
        .unwrap()
        .enumerate_descriptor_bindings(None)
        .unwrap()
}

impl ShaderData {
    pub fn new(vert_src: &str, frag_src: &str) -> ShaderData {
        let vertex = shader::SourceCodeShaderInfo::new(
            vert_src,
            "vert_path",
            shader::ShaderKind::Vertex,
            shader::SourceLanguage::GLSL,
            "main"
        ).precompile().unwrap();
        let fragment = shader::SourceCodeShaderInfo::new(
            frag_src,
            "frag_path",
            shader::ShaderKind::Fragment,
            shader::SourceLanguage::GLSL,
            "main"
        ).precompile().unwrap();
        
        let mut vs_descriptors = get_descriptors(&vertex);
        let mut fs_descriptors = get_descriptors(&fragment);

        let mut uniform_map = Vec::new();
        uniform_map.append(&mut vs_descriptors);
        uniform_map.append(&mut fs_descriptors);

        let shader = rendy::shader::ShaderSetBuilder::default()
            .with_vertex(&vertex).unwrap()
            .with_fragment(&fragment).unwrap();
        let reflection = shader.reflect().unwrap();
        
        ShaderData { shader, reflection, uniform_map }
    }

    pub fn get_uniform(&self, name: &str) -> Result<&ReflectBlockVariable, &str> {
        let mut ret = Err("Could not find the uniform variable");
        // Through each binding
        for v in &self.uniform_map {
            // return the block variable that matches the input name
            match v.block.members.iter().find(|m| m.name == name) {
                Some(var) => {
                    ret = Ok(var);
                    break
                },
                _ => ()
            }
        };
        ret
    }
}

// This is specifically for packing an array of parameters for the actual renderpass
#[derive(Debug)]
pub struct DrawData {
    ubo_index: usize,
    mesh_index: usize,
}

// Correspond each type with its UBO name and std140 alignment size
#[derive(Debug, Clone)]
pub enum UboType {
    //   Type          Name    Offset
    Mat4(Matrix4<f32>, String, u64),
    Vec2(Vector2<f32>, String, u64),
    Vec3(Vector3<f32>, String, u64),
    Vec4(Vector4<f32>, String, u64),
    Float(       f32 , String, u64),
}

#[derive(Debug)]
pub struct UboData {
    // Track the current data of the UBO passed into the scene
    pub current_data: Vec<UboType>,
    // For a single frame, track the changes to the uniform with each draw call, then upload into the buffer during the prepare, and index during the draw
    pub frame_data: Vec<Vec<UboType>>
}

impl UboData {
    pub fn new() -> Self {
        UboData {
            current_data: Vec::new(),
            frame_data: Vec::new()
        }
    }
}

/// Aux Data
#[derive(Debug)]
pub struct Aux<B: hal::Backend> {
    /// Projection transform.
    pub projection: Matrix4<f32>,
    /// Camera transform.
    pub camera: Matrix4<f32>,
    /// Model transform.
    pub model: Matrix4<f32>,
    pub transform_stack: Vec<Matrix4<f32>>,
    /// Uniform data buffer
    pub ubo_data: UboData,
    pub draw_data: Vec<DrawData>,
    pub frames: usize,
    pub align: u64,
    /// Pipeline state selectors
    pub draw_mode: PipelineDrawMode,
    pub cull_mode: PipelineCullMode,
    /// Pass shaders for pipeline creation
    pub bound_shader: usize,
    pub shaders: Vec<ShaderData>,
    /// Active vertex for drawing
    pub bound_vertex_array: usize,
    pub vertex_arrays: Vec<VertexArrayData<B>>,
    /// Texture unit
    pub bound_texture: usize,
    pub textures: Vec<texture::Texture<B>>,
}

impl<B> Aux<B>
where
    B: hal::Backend
{
    pub fn new() -> Aux<B> {
        let frames: usize = 3;
        let align: u64 = 0;
        let bound_shader: usize = 0;
        let shaders = Vec::new();
        let bound_vertex_array: usize = 0;
        let vertex_arrays: Vec<VertexArrayData<B>> = Vec::new();
        let bound_texture: usize = 0;
        let mat_id = mat4_id();
        let draw_mode = PipelineDrawMode::Fill;
        let cull_mode = PipelineCullMode::Off;
        Aux {
            projection: mat_id,
            camera: mat_id,
            model: mat_id,
            transform_stack: vec![],
            ubo_data: UboData::new(),
            draw_data: Vec::new(),
            frames,
            align,
            draw_mode,
            cull_mode,
            bound_shader,
            shaders,
            bound_vertex_array,
            vertex_arrays,
            bound_texture,
            textures: Vec::new(),
        }
    }

    pub fn push_program(&mut self, vert_src: &str, frag_src: &str) {
        self.shaders.push(ShaderData::new(vert_src, frag_src));
    }
}

#[derive(Debug, Default)]
struct RendyPipelineDesc;

impl<B> render::SimpleGraphicsPipelineDesc<B, Aux<B>> for RendyPipelineDesc 
where
    B: hal::Backend
{
    type Pipeline = RendyPipeline<B>;

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        vec![PosColorTexNorm::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
    }

    fn layout(&self) -> render::Layout {
        // This is ignored for a reflection read in the render group
        Layout {
            sets: vec![],
            push_constants: Vec::new(),
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &Aux<B>) -> shader::ShaderSet<B> {
        match _aux.shaders.get(_aux.bound_shader) {
            Some(shader) => shader.shader.build(factory, Default::default()).unwrap(),
            None => panic!("Bound shader cannot be built because it does not exist")
        }
    }

    fn build<'a>(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        aux: &Aux<B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<RendyPipeline<B>, failure::Error> {
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        // Size of the UBO for the attached shader
        let ubo_size = (((size_of::<f32>() as u64 * 53) / align) + 1) * align;
        let frame_size = ubo_size * UBOS_PER_FRAME_SIZE;
        let frames = ctx.frames_in_flight as u64;

        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: frame_size * frames as u64,
                    usage: hal::buffer::Usage::UNIFORM
                },
                Dynamic
            )
            .unwrap();

        let mut sets = Vec::with_capacity(frames as usize * UBOS_PER_FRAME_SIZE as usize);
        for frame in 0..frames {
            for index in 0..UBOS_PER_FRAME_SIZE {
                let frame_offset = frame * frame_size;
                let ubo_offset = ubo_size * index;
                let set = factory
                    .create_descriptor_set(set_layouts[0].clone())
                    .unwrap();
                unsafe {
                    factory.write_descriptor_sets(Some(hal::pso::DescriptorSetWrite {
                        set: set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: Some(hal::pso::Descriptor::Buffer(
                            buffer.raw(),
                            Some(frame_offset + ubo_offset)..Some(frame_offset + ubo_offset + ubo_size)
                        ))
                    }));
                    sets.push(set);
                }
            }
        }

        let mut sampler_sets = Vec::new();
        for texture in &aux.textures {
            let set = factory
                .create_descriptor_set(set_layouts[1].clone())
                .unwrap();
            unsafe {
                factory.device().write_descriptor_sets(Some(
                    hal::pso::DescriptorSetWrite {
                        set: set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: vec![hal::pso::Descriptor::CombinedImageSampler(
                            texture.view().raw(),
                            hal::image::Layout::ShaderReadOnlyOptimal,
                            texture.sampler().raw()
                        )],
                    }
                ));
            }
            sampler_sets.push(set); // NOW SAVE BINDING IN SCENE FUNCTION TO APPLY THIS SET
        }

        // Buffer is arranged in three levels as such:
        // Frames(3) [
        //     Ubos(2048) [
        //         Ubo { .. }, ..
        //     ], ..
        // ]
        Ok(RendyPipeline {
            frame_size, 
            ubo_size, 
            ubo_index_prep: 0, 
            ubo_index_draw: 0, 
            buffer, 
            sets,
            sampler_sets
        })
    }
}

/// Stores state for the rendy backend.
#[derive(Debug)]
struct RendyPipeline<B: hal::Backend> {
    //textures: Vec<rendy::texture::Texture<B>>,
    frame_size: u64, // size of a single frame
    ubo_size: u64, // size of a single UBO
    ubo_index_prep: u64, // selects which UBO within the frame we're using
    ubo_index_draw: usize,
    buffer: Escape<Buffer<B>>,
    sets: Vec<Escape<DescriptorSet<B>>>,
    sampler_sets: Vec<Escape<DescriptorSet<B>>>,
}

impl<B> render::SimpleGraphicsPipeline<B, Aux<B>> for RendyPipeline<B>
where
    B: hal::Backend
{
    type Desc = RendyPipelineDesc;

    // This should execute before each frame
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        index: usize,
        aux: &Aux<B>,
    ) -> PrepareResult {
        // The idea here is to allocate a large buffer to store each UniformData corresponding to the current state when a draw is needed in the FrameGraph
        let frame_offset = self.frame_size * index as u64;
        
        // Define a macro to upload type variable uniforms
        macro_rules! upload {
            ($data:expr) => {
                |_name: &String, uniform_offset: u64| {
                    unsafe {
                        factory
                        .upload_visible_buffer(
                            &mut self.buffer, 
                            uniform_offset, 
                            &[$data.clone()]
                        )
                        .unwrap();
                    }
                }
            }
        };

        // Upload each ubo according to the scheduled draw calls
        for ubo in &aux.ubo_data.frame_data {
            let frame_ubo_offset: u64 = frame_offset + (self.ubo_size * self.ubo_index_prep);
            // Upload each ubo member
            for uniform in ubo {
                match uniform {
                    UboType::Mat4 (u, n, o) => upload!(u)(n, frame_ubo_offset + o),
                    UboType::Vec2 (u, n, o) => upload!(u)(n, frame_ubo_offset + o),
                    UboType::Vec3 (u, n, o) => upload!(u)(n, frame_ubo_offset + o),
                    UboType::Vec4 (u, n, o) => upload!(u)(n, frame_ubo_offset + o),
                    UboType::Float(u, n, o) => upload!(u)(n, frame_ubo_offset + o),
                };
            }
            self.ubo_index_prep = (self.ubo_index_prep + 1) % UBOS_PER_FRAME_SIZE;
        }

        // Don't reuse drawing command buffers
        render::PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        aux: &Aux<B>,
    ) {
        for draw in &aux.draw_data {
            let frame_offset = UBOS_PER_FRAME_SIZE as usize * index;
            let frame_ubo_index: usize = frame_offset + self.ubo_index_draw;

            // Bind uniform and sampler
            let mut bindings = vec![self.sets[frame_ubo_index].raw()];
            if aux.textures.len() > 0 {
                bindings.push(self.sampler_sets[aux.bound_texture].raw());
            }
            unsafe {
                encoder.bind_graphics_descriptor_sets(
                    layout, 
                    0, 
                    bindings, 
                    std::iter::empty()
                );
            };

            // Bind vertex buffer
            let vbuf = &aux.vertex_arrays[draw.mesh_index].buffer;
            let vref = vbuf.as_ref().unwrap();
            vref.bind_and_draw(0, &[PosColorTexNorm::vertex()], 0..1, &mut encoder)
                .unwrap();

            self.ubo_index_draw = (self.ubo_index_draw + 1) % UBOS_PER_FRAME_SIZE as usize;
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Aux<B>) {}
}

// https://github.com/amethyst/rendy/blob/master/graph/src/node/render/group/simple.rs#L396
fn push_vertex_desc(
    elements: &[hal::pso::Element<hal::format::Format>],
    stride: hal::pso::ElemStride,
    rate: hal::pso::VertexInputRate,
    vertex_buffers: &mut Vec<hal::pso::VertexBufferDesc>,
    attributes: &mut Vec<hal::pso::AttributeDesc>,
) {
    let index = vertex_buffers.len() as hal::pso::BufferIndex;

    vertex_buffers.push(hal::pso::VertexBufferDesc {
        binding: index,
        stride,
        rate,
    });

    let mut location = attributes.last().map_or(0, |a| a.location + 1);
    for &element in elements {
        attributes.push(hal::pso::AttributeDesc {
            location,
            binding: index,
            element,
        });
        location += 1;
    }
}

// Implement RenderGroup* to expose GraphicsPipelineDesc

#[derive(Debug)]
struct RendyGroupDesc<P: std::fmt::Debug> {
    inner: P
}

impl RendyGroupDesc<RendyPipelineDesc> {
    pub fn new() -> Self {
        Self { inner: RendyPipelineDesc{} }
    }
}

impl<B, T, P> RenderGroupDesc<B, T> for RendyGroupDesc<P>
where
    B: hal::Backend,
    T: Sized,
    P: SimpleGraphicsPipelineDesc<B, T>,
{
    fn buffers(&self) -> Vec<BufferAccess> {
        self.inner.buffers()
    }

    fn images(&self) -> Vec<ImageAccess> {
        self.inner.images()
    }

    fn colors(&self) -> usize {
        self.inner.colors().len()
    }

    fn depth(&self) -> bool {
        self.inner.depth_stencil().is_some()
    }

    fn build<'a>(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &T,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, T>>, failure::Error> {
        let mut shader_set = self.inner.load_shader_set(factory, aux);

        let pipeline = self.inner.pipeline();

        // Ignore the layout in the pipeline, use the reflection in aux
        let set_layouts = unsafe {
            use std::mem::transmute;
            // reinterpret_cast T to Aux
            let x: &Aux<B> = transmute(aux);
            let layout = match x.shaders.get(x.bound_shader) {
                Some(shader) => shader.reflection.layout().unwrap(),
                None => panic!("Bound shader cannot be built because it does not exist")
            };
            layout
                .sets
                .into_iter()
                .map(|set| {
                    factory.create_descriptor_set_layout(set.bindings).map(Handle::from)
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    shader_set.dispose(factory);
                    e
                })?
        };

        let pipeline_layout = unsafe {
            factory.device().create_pipeline_layout(
                set_layouts.iter().map(|l| l.raw()),
                pipeline.layout.push_constants,
            )
        }
        .map_err(|e| {
            shader_set.dispose(factory);
            e
        })?;

        assert_eq!(pipeline.colors.len(), self.inner.colors().len());

        let mut vertex_buffers = Vec::new();
        let mut attributes = Vec::new();

        for &(ref elemets, stride, rate) in &pipeline.vertices {
            push_vertex_desc(elemets, stride, rate, &mut vertex_buffers, &mut attributes);
        }

        // Invert Y axis to match OpenGL's NDC space
        // Handy charts: https://github.com/gfx-rs/gfx/tree/master/src/backend
        #[cfg(feature = "vulkan")]
        let viewport = hal::pso::Rect {
            x: 0,
            y: framebuffer_height as i16,
            w: framebuffer_width as i16,
            h: -(framebuffer_height as i16),
        };

        // DX12 and Metal both have the same NDC space as OpenGL
        #[cfg(any(feature = "dx12", feature = "metal"))]
        let viewport = hal::pso::Rect {
            x: 0,
            y: 0,
            w: framebuffer_width as i16,
            h: framebuffer_height as i16,
        };

        let scissor = hal::pso::Rect {
            x: 0,
            y: 0,
            w: framebuffer_width as i16,
            h: framebuffer_height as i16,
        };

        let shaders = match shader_set.raw() {
            Err(e) => {
                shader_set.dispose(factory);
                return Err(e);
            }
            Ok(s) => s,
        };

        // https://vulkan-tutorial.com/Multisampling#page_Getting-available-sample-count
        let limits = factory.physical().limits();
        let sample_count: u8 = {
            let counts = std::cmp::min(
                limits.framebuffer_color_samples_count,
                limits.framebuffer_depth_samples_count
            );
            let mut samples = 1;
            if (counts & 64) > 0 { samples = 64; }
            if (counts & 32) > 0 { samples = 32; }
            if (counts & 16) > 0 { samples = 16; }
            if (counts &  8) > 0 { samples =  8; }
            if (counts &  4) > 0 { samples =  4; }
            if (counts &  2) > 0 { samples =  2; }
            samples
        };

        // This doesn't work as of gfx-hal 0.21
        let _multisampling = hal::pso::Multisampling {
            rasterization_samples: sample_count,
            sample_shading: None,
            sample_mask: !0,
            alpha_coverage: false,
            alpha_to_one: false
        };

        let mut gp_desc = hal::pso::GraphicsPipelineDesc {
            shaders,
            rasterizer: hal::pso::Rasterizer::FILL,
            vertex_buffers,
            attributes,
            input_assembler: pipeline.input_assembler_desc,
            blender: hal::pso::BlendDesc {
                logic_op: None,
                targets: pipeline.colors.clone(),
            },
            depth_stencil: pipeline.depth_stencil,
            multisampling: None,
            baked_states: hal::pso::BakedStates {
                viewport: Some(hal::pso::Viewport {
                    rect: viewport,
                    depth: 0.0..1.0,
                }),
                scissor: Some(scissor),
                blend_color: None,
                depth_bounds: None,
            },
            layout: &pipeline_layout,
            subpass,
            flags: hal::pso::PipelineCreationFlags::ALLOW_DERIVATIVES,
            parent: hal::pso::BasePipeline::None,
        };

        let rast_fill = hal::pso::Rasterizer {
            polygon_mode: hal::pso::PolygonMode::Fill,
            cull_face: hal::pso::Face::NONE,
            front_face: hal::pso::FrontFace::CounterClockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        };

        let rast_line = hal::pso::Rasterizer {
            polygon_mode: hal::pso::PolygonMode::Line(1.0),
            cull_face: hal::pso::Face::NONE,
            front_face: hal::pso::FrontFace::CounterClockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        };

        let rast_point = hal::pso::Rasterizer {
            polygon_mode: hal::pso::PolygonMode::Point,
            cull_face: hal::pso::Face::NONE,
            front_face: hal::pso::FrontFace::CounterClockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        };

        // Establish the base pipeline
        let gp_base = unsafe {
            factory.device().create_graphics_pipeline(&gp_desc, None)
        }?;

        macro_rules! set_culls {
            ($rast:ident) => {
                {
                    // Setup derivative pipelines
                    gp_desc.parent = hal::pso::BasePipeline::Pipeline(&gp_base);
                    gp_desc.rasterizer = $rast;
                    gp_desc.rasterizer.cull_face = hal::pso::Face::NONE;
                    let gp_cull_none = unsafe {
                        factory.device().create_graphics_pipeline(&gp_desc, None)
                    }?;
                    gp_desc.rasterizer.cull_face = hal::pso::Face::FRONT;
                    let gp_cull_front = unsafe {
                        factory.device().create_graphics_pipeline(&gp_desc, None)
                    }?;
                    gp_desc.rasterizer.cull_face = hal::pso::Face::BACK;
                    let gp_cull_back = unsafe {
                        factory.device().create_graphics_pipeline(&gp_desc, None)
                    }?;
                    gp_desc.rasterizer.cull_face = hal::pso::Face::FRONT;
                    let gp_cull_front_back = unsafe {
                        factory.device().create_graphics_pipeline(&gp_desc, None)
                    }?;
                    [gp_cull_none, gp_cull_front, gp_cull_back, gp_cull_front_back]
                }
            };
        }

        let fill_culls = set_culls!(rast_fill);
        let line_culls = set_culls!(rast_line);
        let point_culls = set_culls!(rast_point);
        let graphics_pipelines = [fill_culls, line_culls, point_culls];

        let parents = vec![gp_base];

        let pipeline = self
            .inner
            .build(ctx, factory, queue, aux, buffers, images, &set_layouts)
            .map_err(|e| {
                shader_set.dispose(factory);
                e
            })?;

        shader_set.dispose(factory);

        Ok(Box::new(RendyGroup::<B, _> {
            set_layouts,
            pipeline_layout,
            parents,
            graphics_pipelines,
            pipeline,
        }))
    }
}

#[derive(Debug)]
struct RendyGroup<B: hal::Backend, P> {
    set_layouts: Vec<Handle<DescriptorSetLayout<B>>>,
    pipeline_layout: B::PipelineLayout,
    parents: Vec<B::GraphicsPipeline>,
    graphics_pipelines: PipelineModes<B>,
    pipeline: P,
}

impl<B, T, P> RenderGroup<B, T> for RendyGroup<B, P>
where
    B: hal::Backend,
    T: Sized,
    P: SimpleGraphicsPipeline<B, T>,
{
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &T,
    ) -> PrepareResult {
        self.pipeline.prepare(factory, queue, &self.set_layouts, index, aux)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &T,
    ) {
        use std::mem::transmute;
        let (draw_mode, cull_mode) = unsafe {
            // reinterpret_cast T to Aux to access draw_mode and cull_mode
            let x: &Aux<B> = transmute(aux);
            (x.draw_mode as usize, x.cull_mode as usize)
        };
        encoder.bind_graphics_pipeline(&self.graphics_pipelines[draw_mode][cull_mode]);
        self.pipeline.draw(&self.pipeline_layout, encoder, index, aux);
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, aux: &T) {
        use std::ptr::{read};
        self.pipeline.dispose(factory, aux);

        for i in 0..self.graphics_pipelines.len() {
            for j in 0..self.graphics_pipelines[i].len() {
                unsafe {
                    factory.device().destroy_graphics_pipeline(
                        read(&self.graphics_pipelines[i][j])
                    );
                }
            }
        }

        unsafe {
            factory.device().destroy_pipeline_layout(self.pipeline_layout);
            drop(self.set_layouts);
        }
    }
}

struct RendyState<B>
where
    B: hal::Backend
{
    // winit types
    pub window: WinitWindow,
    // Rendy types
    pub graph: Option<rendy::graph::Graph<B, Aux<B>>>,
    pub aux: Aux<B>,
    pub pipelines: Vec<RendyPipeline<B>>,
    // Device
    pub factory: Factory<B>,
    pub queue_id: QueueId,
    pub families: rendy::command::Families<B>,
}

impl<B> RendyState<B>
where
    B: hal::Backend
{
    pub fn new(window_settings: WindowSettings) -> RendyState<B> {
        let window: WinitWindow = WinitWindow::new(&window_settings);

        let config: Config = Default::default();

        let (factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();

        // HACK !
        // https://github.com/ggez/ggraphics/blob/master/src/lib.rs#L1154
        let queue_id = QueueId {
            family: families.family_by_index(0).id(),
            index: 0,
        };

        let aux = Aux::new();
        let pipelines: Vec<RendyPipeline<B>> = Vec::new();
        let graph = None;

        RendyState {
            window,
            graph,
            aux,
            pipelines,
            factory,
            families,
            queue_id,
        }
    }
}

/// Stores scene data.
pub struct Scene {
    /// Scene settings.
    pub settings: SceneSettings,
    shaders: Vec<*const str>,
    // if culling is disabled, save current state for when its enabled
    saved_cull_mode: PipelineCullMode,
    graph_built: bool,
    rendy_state: RendyState<Backend>
}

impl Scene {
    /// Create new scene.
    pub fn new(settings: SceneSettings, window_settings: WindowSettings) -> Scene {
        env_logger::Builder::from_default_env().init();
        let rendy_state = RendyState::new(window_settings);
        Scene {
            settings,
            shaders: vec![],
            saved_cull_mode: PipelineCullMode::Off,
            graph_built: false,
            rendy_state,
        }
    }

    /// Set projection matrix.
    pub fn projection(&mut self, p: Matrix4<f32>) {
        self.rendy_state.aux.projection = p;
    }

    /// Set camera matrix.
    pub fn camera(&mut self, c: Matrix4<f32>) {
        self.rendy_state.aux.camera = c;
    }

    /// Set model matrix.
    pub fn model(&mut self, m: Matrix4<f32>) {
        self.rendy_state.aux.model = m;
    }

    /// Get the created window.
    pub fn raw_window(&mut self) -> &winit::Window {
        &self.rendy_state.window.get_window()
    }

    /// Get the wrapper that contains the window.
    pub fn window(&mut self) -> &mut WinitWindow {
        &mut self.rendy_state.window
    }

    /// Load texture from path.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, image::ImageError> {
        let image_reader = BufReader::new(File::open(path)?);
        let texture_builder = rendy::texture::image::load_from_image(image_reader, Default::default()).unwrap();
        let texture = texture_builder
            .build(
                ImageState {
                    queue: self.rendy_state.queue_id,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                &mut self.rendy_state.factory
            )
            .unwrap();
        let id = self.rendy_state.aux.textures.len();
        self.rendy_state.aux.textures.push(texture);
        Ok(Texture(id))
    }

    /// Create vertex shader from source.
    pub fn vertex_shader(
        &mut self,
        vertex_shader_src: &str
    ) -> Result<VertexShader, String> {
        let id = self.shaders.len();
        self.shaders.push(vertex_shader_src);
        Ok(VertexShader(id))
    }

    /// Create fragment shader from source.
    pub fn fragment_shader(
        &mut self,
        fragment_shader_src: &str
    ) -> Result<FragmentShader, String> {
        let id = self.shaders.len();
        self.shaders.push(fragment_shader_src);
        Ok(FragmentShader(id))
    }

    /// Create program from vertex and fragment shader.
    pub fn program_from_vertex_fragment(
        &mut self,
        vertex_shader: VertexShader,
        fragment_shader: FragmentShader
    ) -> Program {
        // This should implement a graphics pipeline
        // shader and a pipeline are 1:1
        let id = self.rendy_state.aux.shaders.len();
        unsafe {
            self.rendy_state.aux.push_program(
                &*self.shaders[vertex_shader.0],
                &*self.shaders[fragment_shader.0]
            );
        };
        Program(id)
    }

    /// Create 4D matrix uniform.
    pub fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String> {
        let id = self.rendy_state.aux.ubo_data.current_data.len();
        let uniform = self.rendy_state.aux.shaders[program.0].get_uniform(name)?;
        let m4 = UboType::Mat4(mat4_id(), uniform.name.clone(), uniform.offset as u64);
        self.rendy_state.aux.ubo_data.current_data.push(m4);
        Ok(Matrix4Uniform(id))
    }

    /// Create 2D vector uniform.
    pub fn vector2_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector2Uniform, String> {
        let id = self.rendy_state.aux.ubo_data.current_data.len();
        let uniform = self.rendy_state.aux.shaders[program.0].get_uniform(name)?;
        let zero: f32 = 0.0;
        let v2 = UboType::Vec2([zero, zero], uniform.name.clone(), uniform.offset as u64);
        self.rendy_state.aux.ubo_data.current_data.push(v2);
        Ok(Vector2Uniform(id))
    }

    /// Create 3D vector uniform.
    pub fn vector3_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<Vector3Uniform, String> {
        let id = self.rendy_state.aux.ubo_data.current_data.len();
        let uniform = self.rendy_state.aux.shaders[program.0].get_uniform(name)?;
        let zero: f32 = 0.0;
        let v3 = UboType::Vec3([zero, zero, zero], uniform.name.clone(), uniform.offset as u64);
        self.rendy_state.aux.ubo_data.current_data.push(v3);
        Ok(Vector3Uniform(id))
    }

    /// Create f32 uniform.
    pub fn f32_uniform(
        &mut self,
        program: Program,
        name: &str
    ) -> Result<F32Uniform, String> {
        let id = self.rendy_state.aux.ubo_data.current_data.len();
        let uniform = self.rendy_state.aux.shaders[program.0].get_uniform(name)?;
        let zero: f32 = 0.0;
        let f = UboType::Float(zero, uniform.name.clone(), uniform.offset as u64);
        self.rendy_state.aux.ubo_data.current_data.push(f);
        Ok(F32Uniform(id))
    }

    /// Create vertex array.
    pub fn vertex_array(&mut self) -> VertexArray {
        let id = self.rendy_state.aux.vertex_arrays.len();
        self.rendy_state.aux.vertex_arrays.push(VertexArrayData::new());
        VertexArray(id)
    }

    /// Create uv buffer.
    pub fn uv_buffer(
        &mut self,
        vertex_array: VertexArray,
        _attribute: u32,
        data: &[f32]
    ) -> UVBuffer {
        let id = vertex_array.0;
        self.rendy_state.aux.vertex_arrays[id].add_tex_coords(data);
        UVBuffer(0, 0)
    }

    /// Create color buffer.
    pub fn color_buffer(
        &mut self,
        vertex_array: VertexArray,
        _attribute: u32,
        data: &[f32]
    ) -> ColorBuffer {
        let id = vertex_array.0;
        self.rendy_state.aux.vertex_arrays[id].add_colors(data);
        ColorBuffer(id, 0)
    }

    /// Create vertex buffer for 2D coordinates.
    pub fn vertex_buffer2(
        &mut self,
        _vertex_array: VertexArray,
        _attribute: u32,
        _data: &[f32]
    ) -> VertexBuffer2 {
        // Does nothing for now.
        // let id = vertex_array.0;
        // self.rendy_state.aux.vertex_arrays[id].add_positions(data);
        VertexBuffer2(0, 0)
    }

    /// Create vertex buffer for 3D coordinates.
    pub fn vertex_buffer3(
        &mut self,
        vertex_array: VertexArray,
        _attribute: u32,
        data: &[f32]
    ) -> VertexBuffer3 {
        let id = vertex_array.0;
        self.rendy_state.aux.vertex_arrays[id].add_positions(data);
        VertexBuffer3(id, 0)
    }

    /// Create normal buffer.
    pub fn normal_buffer(
        &mut self,
        vertex_array: VertexArray,
        _attribute: u32,
        data: &[f32]
    ) -> NormalBuffer {
        let id = vertex_array.0;
        self.rendy_state.aux.vertex_arrays[id].add_normals(data);
        NormalBuffer(id, 0)
    }

    /// Use program.
    pub fn use_program(&mut self, program: Program) {
        // Binding the pipeline that holds the program
        self.rendy_state.aux.bound_shader = program.0;
    }

    /// Set model-view-projection transform uniform.
    pub fn set_model_view_projection(&mut self, matrix_id: Matrix4Uniform) {
        use vecmath::col_mat4_mul as mul;
        let mvp = mul(
            mul(self.rendy_state.aux.projection, self.rendy_state.aux.camera), 
            self.rendy_state.aux.model
        );
        match self.rendy_state.aux.ubo_data.current_data.get_mut(matrix_id.0) {
            Some(UboType::Mat4(data, _, _)) => *data = mvp,
            _ => panic!("The matrix uniform must exist before you set it as MVP")
        }
    }

    /// Set view transform uniform.
    pub fn set_view(&mut self, matrix_id: Matrix4Uniform) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(matrix_id.0) {
            Some(UboType::Mat4(data, _, _)) => *data = self.rendy_state.aux.camera,
            _ => panic!("The matrix uniform must exist before you set it as view")
        }
    }

    /// Set model transform uniform.
    pub fn set_model(&mut self, matrix_id: Matrix4Uniform) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(matrix_id.0) {
            Some(UboType::Mat4(data, _, _)) => *data = self.rendy_state.aux.model,
            _ => panic!("The matrix uniform must exist before you set it as model")
        }
    }

    /// Set matrix uniform.
    pub fn set_matrix4(&mut self, matrix_id: Matrix4Uniform, val: Matrix4<f32>) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(matrix_id.0) {
            Some(UboType::Mat4(data, _, _)) => *data = val,
            _ => panic!("The Matrix4Uniform does not exist")
        }
    }

    /// Set 2D vector uniform.
    pub fn set_vector2(&mut self, v_id: Vector2Uniform, v: Vector2<f32>) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(v_id.0) {
            Some(UboType::Vec2(data, _, _)) => *data = v,
            _ => panic!("The Vector2Uniform does not exist")
        }
    }

    /// Set 3D vector uniform.
    pub fn set_vector3(&mut self, v_id: Vector3Uniform, v: Vector3<f32>) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(v_id.0) {
            Some(UboType::Vec3(data, _, _)) => *data = v,
            _ => panic!("The Vector2Uniform does not exist")
        }
    }

    /// Set f32 uniform.
    pub fn set_f32(&mut self, f_id: F32Uniform, v: f32) {
        match self.rendy_state.aux.ubo_data.current_data.get_mut(f_id.0) {
            Some(UboType::Float(data, _, _)) => *data = v,
            _ => panic!("The Vector2Uniform does not exist")
        }
    }

    /// Set texture.
    pub fn set_texture(&mut self, texture_id: Texture) {
        self.rendy_state.aux.bound_texture = texture_id.0;
    }

    /// Translate model.
    pub fn translate(&mut self, v: Vector3<f32>) {
        let mat = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [v[0], v[1], v[2], 1.0]
        ];
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
    }

    /// Translate model in global coordinates.
    pub fn translate_global(&mut self, v: Vector3<f32>) {
        for i in 0..3 {
            self.rendy_state.aux.model[3][i] += v[i];
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
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
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
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
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
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
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
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
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
        self.rendy_state.aux.model = col_mat4_mul(self.rendy_state.aux.model, mat);
    }

    /// Push model transfrom to transform stack.
    pub fn push_transform(&mut self) {
        self.rendy_state.aux.transform_stack.push(self.rendy_state.aux.model);
    }

    /// Pop model transform from transform stack.
    pub fn pop_transform(&mut self) {
        if let Some(mat) = self.rendy_state.aux.transform_stack.pop() {
            self.rendy_state.aux.model = mat;
        }
    }

    /// Clear background with color.
    pub fn clear(&self, _bg_color: [f32; 4]) {}

    /// Enable cull face.
    pub fn enable_cull_face(&mut self) {
        self.rendy_state.aux.cull_mode = self.saved_cull_mode;
    }

    /// Disable cull face.
    pub fn disable_cull_face(&mut self) {
        self.saved_cull_mode = self.rendy_state.aux.cull_mode;
        self.rendy_state.aux.cull_mode = PipelineCullMode::Off;
    }

    /// Cull front face.
    pub fn cull_face_front(&mut self) {
        self.rendy_state.aux.cull_mode = PipelineCullMode::Front;
    }

    /// Cull back face.
    pub fn cull_face_back(&mut self) {
        self.rendy_state.aux.cull_mode = PipelineCullMode::Back;
    }

    /// Cull both front and back face.
    pub fn cull_face_front_and_back(&mut self) {
        self.rendy_state.aux.cull_mode = PipelineCullMode::FrontBack;
    }

    /// Draws triangles.
    pub fn draw_triangles(&mut self, vertex_array: VertexArray, _len: usize) {
        // Set the correct pipeline mode
        self.rendy_state.aux.draw_mode = PipelineDrawMode::Fill;
        // Bind the vertex array for drawing
        self.rendy_state.aux.bound_vertex_array = vertex_array.0;
        let mesh_index = vertex_array.0;
        let ubo_index = self.rendy_state.aux.ubo_data.frame_data.len();
        let fd = self.rendy_state.aux.ubo_data.current_data.clone();
        // Push the state of ubo and mesh binding for later drawing
        self.rendy_state.aux.ubo_data.frame_data.push(fd);
        self.rendy_state.aux.draw_data.push(DrawData {mesh_index, ubo_index});
    }

    /// Draws triangles.
    pub fn draw_lines(&mut self, vertex_array: VertexArray, _len: usize) {
        // Set the correct pipeline mode
        self.rendy_state.aux.draw_mode = PipelineDrawMode::Line;
        // Bind the vertex array for drawing
        self.rendy_state.aux.bound_vertex_array = vertex_array.0;
        let mesh_index = vertex_array.0;
        let ubo_index = self.rendy_state.aux.ubo_data.frame_data.len();
        let fd = self.rendy_state.aux.ubo_data.current_data.clone();
        // Push the state of ubo and mesh binding for later drawing
        self.rendy_state.aux.ubo_data.frame_data.push(fd);
        self.rendy_state.aux.draw_data.push(DrawData {mesh_index, ubo_index});
    }

    /// Draws triangles.
    pub fn draw_points(&mut self, vertex_array: VertexArray, _len: usize) {
        // Set the correct pipeline mode
        self.rendy_state.aux.draw_mode = PipelineDrawMode::Point;
        // Bind the vertex array for drawing
        self.rendy_state.aux.bound_vertex_array = vertex_array.0;
        let mesh_index = vertex_array.0;
        let ubo_index = self.rendy_state.aux.ubo_data.frame_data.len();
        let fd = self.rendy_state.aux.ubo_data.current_data.clone();
        // Push the state of ubo and mesh binding for later drawing
        self.rendy_state.aux.ubo_data.frame_data.push(fd);
        self.rendy_state.aux.draw_data.push(DrawData {mesh_index, ubo_index});
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
                //EnableFrameBufferSRGB => self.enable_framebuffer_srgb(), //-not simple
                //DisableFrameBufferSRGB => self.disable_framebuffer_srgb(), // not simple
                //EnableBlend => self.enable_blend(), //-simple w/ gfx_hal::command::RawCommandBuffer::set_scissors(self.raw, first_scissor, rects)
                //DisableBlend => self.disable_blend(), // simple
                EnableCullFace => self.enable_cull_face(),
                DisableCullFace => self.disable_cull_face(),
                CullFaceFront => self.cull_face_front(),
                CullFaceBack => self.cull_face_back(),
                CullFaceFrontAndBack => self.cull_face_front_and_back(),
                DrawTriangles(vertex_array, len) => self.draw_triangles(vertex_array, len),
                DrawTriangleStrip(vertex_array, len) => self.draw_triangles(vertex_array, len), // directs to normal triangles for the moment
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
                Draw(command_list) => self.draw_recursive(command_list, frame_graph),
                _ => ()
            }
        }
    }

    /// Use this for recursive draw calls, the main draw function executes all the draw commands for a whole frame!
    fn draw_recursive(&mut self, command_list: CommandList, frame_graph: &FrameGraph) {
        self.submit(&frame_graph.command_lists[command_list.0], frame_graph)
    }

    fn build_graph(&mut self) -> Graph<Backend, Aux<Backend>> {
        let surface = self.rendy_state.factory.create_surface(self.rendy_state.window.get_window());
        let mut graph_builder = GraphBuilder::<Backend, Aux<Backend>>::new();

        let size = self.rendy_state.window.get_window()
            .get_inner_size()
            .unwrap()
            .to_physical(self.rendy_state.window.get_window().get_hidpi_factor());
        let window_kind = hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1);

        let color = graph_builder.create_image(
            window_kind,
            1,
            self.rendy_state.factory.get_surface_format(&surface),
            Some(hal::command::ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            hal::format::Format::D16Unorm,
            Some(hal::command::ClearValue::DepthStencil(
                hal::command::ClearDepthStencil(1.0, 0),
            )),
        );

        let render_group_desc = RendyGroupDesc::new();
        let pass = graph_builder.add_node(
            render_group_desc.builder()
                .into_subpass()
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass()
        );

        let present_builder = PresentNode::builder(&self.rendy_state.factory, surface, color)
            .with_dependency(pass);
        let frames = present_builder.image_count();

        graph_builder.add_node(present_builder);
        graph_builder
            .with_frames_in_flight(frames)
            .build(&mut self.rendy_state.factory, &mut self.rendy_state.families, &self.rendy_state.aux)
            .unwrap()
    }

    /// Draws a command list from frame graph.
    pub fn draw(&mut self, command_list: CommandList, frame_graph: &FrameGraph) {
        self.submit(&frame_graph.command_lists[command_list.0], frame_graph);

        // check if all the meshes have uploaded
        for mesh in &mut self.rendy_state.aux.vertex_arrays {
            mesh.upload_check(&self.rendy_state.factory, self.rendy_state.queue_id);
        }

        // build the graph
        if !self.graph_built {
            self.rendy_state.graph = Some(self.build_graph());
            self.graph_built = true;
        }
        
        // Execute the draw
        self.rendy_state.factory.maintain(&mut self.rendy_state.families);

        // Shader binding is associated with the drawing graph
        self.rendy_state.graph.as_mut().unwrap().run(
            &mut self.rendy_state.factory, 
            &mut self.rendy_state.families, 
            &self.rendy_state.aux
        );
        
        // reset ubo draw state
        self.rendy_state.aux.ubo_data.frame_data.clear();
        self.rendy_state.aux.draw_data.clear();
    }

    /// Drop trait does not move self, which is required, so drop needs to be called explicitly
    pub fn drop(mut self) {
        self.rendy_state.graph.unwrap().dispose(&mut self.rendy_state.factory, &self.rendy_state.aux);
    }
}
