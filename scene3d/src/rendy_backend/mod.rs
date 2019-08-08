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
    mem::size_of,
    sync::Arc
};
use self::rendy::{
    command::{RenderPassEncoder, QueueId},
    factory::{Config, Factory},
    graph::{render, GraphContext, NodeBuffer, NodeImage, GraphBuilder, present::PresentNode},
    hal::{self, PhysicalDevice, Device},
    resource::{DescriptorSet, Buffer, Escape, DescriptorSetLayout, Handle, BufferInfo},
    wsi::winit,
    mesh::{Position, Color, TexCoord, Normal, VertexFormat, Mesh},
    memory::{Dynamic},
    shader
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
}

impl<B> VertexArrayData<B> 
where
    B: hal::Backend
{
    pub fn new() -> VertexArrayData<B> {
        VertexArrayData {
            buffer: None,
            vertices: None
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
            vertex.tex_coord = TexCoord([tex[0], tex[1]]);
        }
    }

    pub fn add_normals(&mut self, data: &[f32]) {
        if self.vertices.is_none() {
            self.reset(data.len()/2)
        }
        let mut norm_chunks = data.chunks(2);
        for vertex in self.vertices.as_mut().unwrap() {
            let norm = norm_chunks.next().unwrap();
            vertex.normal = Normal([norm[0], norm[1], norm[2]]);
        }
    }

    pub fn upload_check(&mut self, factory: &Factory<B>, queue: QueueId) {
        if self.vertices.is_none() {
            return
        }
        else {
            self.buffer = Some(
                Mesh::<B>::builder()
                    .with_vertices(&self.vertices.as_ref().unwrap()[..])
                    .build(queue, &factory)
                    .unwrap()
            );
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

    pub fn get_uniform(&self, name: &str) -> Vec<&ReflectBlockVariable> {
        let mut variables: Vec<&ReflectBlockVariable> = Vec::new();
        // Through each binding
        for v in &self.uniform_map {
            // push a block variable that matches the input name
            match v.block.members.iter().find(|m| m.name == name) {
                Some(var) => variables.push(var),
                _ => ()
            }
        };
        variables
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
    // Used to perform drawing and transformation in a pipeline draw
    pub frame_graph: Arc<FrameGraph>,
    pub bound_command_list: CommandList,
    // 
    pub frames: usize,
    pub align: u64,
    // Pass shaders for pipeline creation
    pub bound_shader: usize,
    pub shaders: Vec<ShaderData>,
    // Active vertex for drawing
    pub bound_vertex_array: usize,
    pub vertex_arrays: Vec<VertexArrayData<B>>
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
        let mat_id = mat4_id();
        Aux {
            projection: mat_id,
            camera: mat_id,
            model: mat_id,
            transform_stack: vec![],
            frame_graph: Arc::new(FrameGraph::new()),
            bound_command_list: CommandList(0),
            frames,
            align,
            bound_shader,
            shaders,
            bound_vertex_array,
            vertex_arrays
        }
    }

    pub fn push_program(&mut self, vert_src: &str, frag_src: &str) {
        self.shaders.push(ShaderData::new(vert_src, frag_src));
    }
}

#[derive(Debug, Default)]
struct RendyPipelineDesc;

impl RendyPipelineDesc {
    fn new() -> RendyPipelineDesc {
        Default::default()
    }
}

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
        return Layout {
            sets: vec![SetLayout {
                bindings: vec![hal::pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: hal::pso::DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::GRAPHICS,
                    immutable_samplers: false,
                }],
            }],
            push_constants: Vec::new(),
        };
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
        _aux: &Aux<B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<RendyPipeline<B>, failure::Error> {
        let size = size_of::<f32>() as u64 * 16;
        let frames = ctx.frames_in_flight as u64;
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;
        // Buffer frame size needs to be a multiple of the min_uniform_buffer_offset_alignment
        let frame_size = ((size / align) + 1) * align;
        print!("{:#?}", frame_size);

        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: frame_size * frames as u64,
                    usage: hal::buffer::Usage::UNIFORM
                },
                Dynamic
            )
            .unwrap();

        let mut sets = Vec::new();
        for index in 0..frames {
            unsafe {
                let set = factory
                    .create_descriptor_set(set_layouts[0].clone())
                    .unwrap();
                factory.write_descriptor_sets(Some(hal::pso::DescriptorSetWrite {
                    set: set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(hal::pso::Descriptor::Buffer(
                        buffer.raw(),
                        Some(index*frame_size)..Some((index*frame_size) + size)
                    ))
                }));
                sets.push(set);
            }
        }

        Ok(RendyPipeline{ align, buffer, sets })
    }
}

/// Stores state for the rendy backend.
#[derive(Debug)]
struct RendyPipeline<B: hal::Backend> {
    //textures: Vec<rendy::texture::Texture<B>>,
    align: u64,
    buffer: Escape<Buffer<B>>,
    sets: Vec<Escape<DescriptorSet<B>>>,
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
        use vecmath::col_mat4_mul as mul;
        unsafe {
            let mvp = mul(mul(aux.projection, aux.camera), aux.model);
            factory
                .upload_visible_buffer(
                    &mut self.buffer, 
                    self.align*index as u64, 
                    &[mvp]
                )
                .unwrap();
        };

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
        // Bind uniform
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout, 
                0, 
                Some(self.sets[index].raw()), 
                std::iter::empty()
            );
        };

        // Bind vertex buffer
        let vbuf = &aux.vertex_arrays[aux.bound_vertex_array].buffer;
        vbuf
            .as_ref()
            .unwrap()
            .bind_and_draw(0, &[PosColorTexNorm::vertex()], 0..1, &mut encoder)
            .unwrap();
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Aux<B>) {}
}

struct RendyState<B>
where
    B: hal::Backend
{
    // winit types
    pub window: WinitWindow,
    // Rendy types
    pub graphs: Vec<rendy::graph::Graph<B, Aux<B>>>,
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
        let graphs = Vec::new();

        RendyState {
            window,
            graphs,
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
    programs: Vec<gl::types::GLuint>,
    uniforms: Vec<gl::types::GLuint>,
    vertex_arrays: Vec<gl::types::GLuint>,
    buffers: Vec<gl::types::GLuint>,
    textures: Vec<gl::types::GLuint>,
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
            programs: vec![],
            uniforms: vec![],
            vertex_arrays: vec![],
            buffers: vec![],
            textures: vec![],
            rendy_state,
        }
    }

    /// Set projection matrix.
    pub fn set_projection(&mut self, p: Matrix4<f32>) {
        self.rendy_state.aux.projection = p;
    }

    /// Set camera matrix.
    pub fn set_camera(&mut self, c: Matrix4<f32>) {
        self.rendy_state.aux.camera = c;
    }

    /// Set model matrix.
    pub fn set_model(&mut self, m: Matrix4<f32>) {
        self.rendy_state.aux.model = m;
    }

    /// Get the created window.
    pub fn get_window(&mut self) -> &winit::Window {
        &self.rendy_state.window.get_window()
    }

    /// asdf
    pub fn get_window_wrapper(&mut self) -> &mut WinitWindow {
        &mut self.rendy_state.window
    }

    /// Load texture from path.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, image::ImageError> {
        let image_reader = BufReader::new(File::open(path)?);
        let _texture_builder = rendy::texture::image::load_from_image(image_reader, Default::default()).unwrap();
        
        Ok(Texture(0))
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
        let id = self.programs.len();
        unsafe {
            self.rendy_state.aux.push_program(
                &*self.shaders[vertex_shader.0],
                &*self.shaders[fragment_shader.0]
            );
        };

        // For now each pipeline is going to be a graph of its own, probably not ideal
        let surface = self.rendy_state.factory.create_surface(self.rendy_state.window.get_window());
        let graph = {
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
                Some(hal::command::ClearValue::Color([0.1, 0.2, 0.3, 1.0].into())),
            );

            let depth = graph_builder.create_image(
                window_kind,
                1,
                hal::format::Format::D16Unorm,
                Some(hal::command::ClearValue::DepthStencil(
                    hal::command::ClearDepthStencil(1.0, 0),
                )),
            );

            let pass = graph_builder.add_node(
                RendyPipeline::builder()
                    .into_subpass()
                    .with_color(color)
                    .with_depth_stencil(depth)
                    .into_pass()
            );

            let present_builder = PresentNode::builder(&self.rendy_state.factory, surface, color).with_dependency(pass);
            let frames = present_builder.image_count();

            graph_builder.add_node(present_builder);
            graph_builder
                .with_frames_in_flight(frames)
                .build(&mut self.rendy_state.factory, &mut self.rendy_state.families, &self.rendy_state.aux)
                .unwrap()
        };

        self.rendy_state.graphs.push(graph);

        Program(id)
    }

    /// Create 4D matrix uniform.
    pub fn matrix4_uniform(
        &mut self,
        program: Program,
        name: &str,
    ) -> Result<Matrix4Uniform, String> {
        let id = self.uniforms.len();
        let _uniform = self.rendy_state.aux.shaders[program.0].get_uniform(name);
        
        //self.uniforms.push(uniform_location);
        Ok(Matrix4Uniform(id))
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
        self.rendy_state.aux.vertex_arrays[id].upload_check(&self.rendy_state.factory, self.rendy_state.queue_id);
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
    pub fn set_model_view_projection(&self, _matrix_id: Matrix4Uniform) {
        use vecmath::col_mat4_mul as mul;
        let _mvp = mul(
            mul(self.rendy_state.aux.projection, self.rendy_state.aux.camera), 
            self.rendy_state.aux.model
        );
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

    /// Draws triangles.
    pub fn draw_triangles(&mut self, vertex_array: VertexArray, _len: usize) {
        let index = self.rendy_state.aux.bound_shader;

        // Bind the vertex array for drawing
        self.rendy_state.aux.bound_vertex_array = vertex_array.0;

        // Execute the draw
        self.rendy_state.factory.maintain(&mut self.rendy_state.families);

        // Shader binding is associated with the drawing graph
        self.rendy_state.graphs[index].run(
            &mut self.rendy_state.factory, 
            &mut self.rendy_state.families, 
            &self.rendy_state.aux
        );
    }

    /// Executes commands in command list.
    pub fn submit(&mut self, commands: &[Command], frame_graph: &FrameGraph) {
        use Command::*;

        for command in commands {
            match *command {
                UseProgram(program) => self.use_program(program),
                SetModelViewProjection(mvp) => self.set_model_view_projection(mvp),
                //SetModel(model) => self.set_model(model),
                //SetView(view) => self.set_view(view),
                //SetTexture(texture) => self.set_texture(texture),
                //SetF32(uni, val) => self.set_f32(uni, val),
                //SetVector2(uni, val) => self.set_vector2(uni, val),
                //SetVector3(uni, val) => self.set_vector3(uni, val),
                //SetMatrix4(uni, val) => self.set_matrix4(uni, val),
                //EnableFrameBufferSRGB => self.enable_framebuffer_srgb(),
                //DisableFrameBufferSRGB => self.disable_framebuffer_srgb(),
                //EnableBlend => self.enable_blend(),
                //DisableBlend => self.disable_blend(),
                //EnableCullFace => self.enable_cull_face(),
                //DisableCullFace => self.disable_cull_face(),
                //CullFaceFront => self.cull_face_front(),
                //CullFaceBack => self.cull_face_back(),
                //CullFaceFrontAndBack => self.cull_face_front_and_back(),
                DrawTriangles(vertex_array, len) => self.draw_triangles(vertex_array, len),
                //DrawTriangleStrip(vertex_array, len) => self.draw_triangle_strip(vertex_array, len),
                //DrawLines(vertex_array, len) => self.draw_lines(vertex_array, len),
                //DrawPoints(vertex_array, len) => self.draw_points(vertex_array, len),
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
                _ => ()
            }
        }
    }

    /// Draws a command list from frame graph.
    pub fn draw(&mut self, command_list: CommandList, frame_graph: &FrameGraph) {
        self.submit(&frame_graph.command_lists[command_list.0], frame_graph)
    }

    /// Drop trait does not move self, which is required
    pub fn drop(mut self) {
        for graph in self.rendy_state.graphs {
            graph.dispose(&mut self.rendy_state.factory, &self.rendy_state.aux);
        }

        drop(self.rendy_state.families);
        drop(self.rendy_state.factory);
    }
}
