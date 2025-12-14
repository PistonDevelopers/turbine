//! Utility functions for WGPU.

use wgpu::{
    BindGroup,
    BindGroupLayout,
    Buffer,
    ColorTargetState,
    DepthStencilState,
    Device,
    FragmentState,
    MultisampleState,
    PipelineLayout,
    PrimitiveState,
    RenderPipeline,
    ShaderModule,
    VertexBufferLayout,
    VertexState,
    util::DeviceExt,
};

/// Create vertex state.
pub fn vertex_state<'a>(
    shader_module: &'a ShaderModule,
    vertex_buffer_layouts: &'a [VertexBufferLayout<'a>],
) -> VertexState<'a> {
    VertexState {
        module: &shader_module,
        entry_point: None,
        buffers: vertex_buffer_layouts,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

/// Create fragment state.
pub fn fragment_state<'a>(
    shader_module: &'a ShaderModule,
    color_targets: &'a [Option<ColorTargetState>],
) -> FragmentState<'a> {
    FragmentState {
        module: &shader_module,
        entry_point: None,
        targets: color_targets,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

/// Create color target state.
pub fn color_target_state_replace(
    surface_config: &wgpu::SurfaceConfiguration,
    blend: Option<wgpu::BlendState>,
) -> wgpu::ColorTargetState {
    ColorTargetState {
        format: surface_config.format,
        blend,
        write_mask: wgpu::ColorWrites::ALL,
    }}

/// Create f32 uniform buffer.
pub fn f32_uniform_buffer(s: f32, device: &Device) -> Buffer {
    use wgpu::BufferUsages as Bu;
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[s]),
            usage: Bu::UNIFORM | Bu::COPY_DST,
        }
    )
}

/// Create matrix4 uniform buffer.
pub fn matrix4_uniform_buffer(mvp: [[f32; 4]; 4], device: &Device) -> Buffer {
    use wgpu::BufferUsages as Bu;
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[mvp]),
            usage: Bu::UNIFORM | Bu::COPY_DST,
        }
    )
}

/// Create vector2 uniform buffer.
pub fn vector2_uniform_buffer(v: [f32; 2], device: &Device) -> Buffer {
    use wgpu::BufferUsages as Bu;
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[v]),
            usage: Bu::UNIFORM | Bu::COPY_DST,
        }
    )
}

/// Create vector3 uniform buffer.
pub fn vector3_uniform_buffer(v: [f32; 3], device: &Device) -> Buffer {
    use wgpu::BufferUsages as Bu;
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[v]),
            usage: Bu::UNIFORM | Bu::COPY_DST,
        }
    )
}

/// Create uniform bind group layout.
pub fn uniform_bind_group_layout(binding: u32, device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform_bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    })
}

/// Create uniform bind group.
pub fn uniform_bind_group(
    uniform_layout: &BindGroupLayout,
    entries: &[wgpu::BindGroupEntry],
    device: &Device,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform_bind_group"),
        layout: uniform_layout,
        entries,
    })
}

/// Create texture bind group layout.
pub fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Initialize surface cofiguration.
pub fn init_surface_config<S: Into<[u32; 2]>>(
    draw_size: S,
) -> wgpu::SurfaceConfiguration {
    let draw_size = draw_size.into();
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: draw_size[0],
        height: draw_size[1],
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
        view_formats: vec![],
        desired_maximum_frame_latency: Default::default(),
    }
}

/// Create depth texture.
pub fn create_depth_texture_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// Create a primitive state.
pub fn primitive_state(
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
) -> PrimitiveState {
    PrimitiveState {
        topology,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode,
        polygon_mode,
        unclipped_depth: false,
        conservative: false,
    }
}

/// Create a depth stencil state.
pub fn depth_stencil_state() -> DepthStencilState {
    DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

/// Create a multisample state.
pub fn multisample_state() -> MultisampleState {
    wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}

/// Create a render pipeline.
pub fn render_pipeline(
    pipeline_layout: &PipelineLayout,
    vs: &wgpu::ShaderModule,
    fs: &wgpu::ShaderModule,
    vertex_buffer_layouts: &[VertexBufferLayout],
    color_targets: &[Option<ColorTargetState>],
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
    device: &Device,
) -> RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(pipeline_layout),
        vertex: vertex_state(vs, vertex_buffer_layouts),
        fragment: Some(fragment_state(fs, color_targets)),
        primitive: primitive_state(topology, polygon_mode, cull_mode),
        depth_stencil: Some(depth_stencil_state()),
        multisample: multisample_state(),
        multiview: None,
        cache: None,
    })
}
