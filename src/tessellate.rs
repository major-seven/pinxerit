use wgpu::{Device, Queue, SurfaceConfiguration};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TessellateVertex {
    color: [f32; 4],
    position: [f32; 3],
}

impl TessellateVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TessellateVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct TessellatePipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub staging_belt: wgpu::util::StagingBelt,
}

pub fn create_tessellate_pipeline(
    device: &Device,
    queue: &Queue,
    config: &SurfaceConfiguration,
    
) -> TessellatePipeline {

    let tessellate_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Tesselate Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tessellate_shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Tessellate Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Tessellate Render Pipeline"),
        layout: Some(&render_pipeline_layout),

        vertex: wgpu::VertexState {
            module: &tessellate_shader,
            entry_point: "vs_main",
            buffers: &[TessellateVertex::desc()],
        },

        fragment: Some(wgpu::FragmentState {
            module: &tessellate_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },

        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    });

    // TODO: maybe size must not be 0 at creation
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tessellate Vertex Buffer"),
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
        size: 0,
    });
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tessellate Index Buffer"),
        usage: wgpu::BufferUsages::INDEX,
        mapped_at_creation: false,
        size: 0,
    });

    let staging_belt = wgpu::util::StagingBelt::new(1024);

    TessellatePipeline {
        render_pipeline,
        vertex_buffer,
        index_buffer,
        num_indices: 0,
        staging_belt,
    }
}
