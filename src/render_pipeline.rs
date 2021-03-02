use crate::texture;
use wgpu::ShaderModuleSource;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline
}

impl RenderPipeline {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, name: &str, vertex_shader: wgpu::ShaderModuleSource, fragment_shader: wgpu::ShaderModuleSource) -> Self {
        // Create the modules
        let vs_module = device.create_shader_module(vertex_shader);
        let fs_module = device.create_shader_module(fragment_shader);

        // Pipeline layout
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        // Create the actual pipeline
        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(
                    wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::Back,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                        clamp_depth: false,
                    }
                ),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[
                    wgpu::ColorStateDescriptor {
                        format: sc_desc.format,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ],
                depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilStateDescriptor::default(),
                }),
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false
            });

        Self { pipeline }
    }
}