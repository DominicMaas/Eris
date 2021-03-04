use crate::texture;
use anyhow::*;
use crate::utils::Vertex;

pub struct RenderPipelineBuilder<'a> {
    layout: Option<&'a wgpu::PipelineLayout>,
    vertex_shader_source: Option<wgpu::ShaderModuleSource<'a>>,
    fragment_shader_source: Option<wgpu::ShaderModuleSource<'a>>,
    texture_format: wgpu::TextureFormat,
    pipeline_name: &'a str
}
impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(texture_format: wgpu::TextureFormat, pipeline_name: &'a str) -> RenderPipelineBuilder {
        Self {
            layout: None,
            vertex_shader_source: None,
            fragment_shader_source: None,
            texture_format,
            pipeline_name
        }
    }

    pub fn with_layout(&mut self, layout: &'a wgpu::PipelineLayout) ->& mut Self {
        self.layout = Some(layout);
        self
    }

    pub fn with_vertex_shader(&mut self, vertex_shader: wgpu::ShaderModuleSource<'a>) -> &mut Self {
        self.vertex_shader_source = Some(vertex_shader);
        self
    }

    pub fn with_fragment_shader(&mut self, fragment_shader: wgpu::ShaderModuleSource<'a>) -> &mut Self {
        self.fragment_shader_source = Some(fragment_shader);
        self
    }

    pub fn build(&mut self, device: &wgpu::Device) -> Result<wgpu::RenderPipeline> {
        // Ensure layout
        if self.layout.is_none() {
            bail!("No pipeline layout was supplied!");
        }
        let layout = self.layout.unwrap();

        // Ensure vertex
        if self.vertex_shader_source.is_none() {
            bail!("No vertex shader supplied!");
        }

        // Ensure fragment
        if self.fragment_shader_source.is_none() {
            bail!("No fragment shader supplied!");
        }

        // Create the modules
        let vs_module = device.create_shader_module(self.vertex_shader_source
            .take().context("Please include a vertex shader")?);
        let fs_module = device.create_shader_module(self.fragment_shader_source
            .take().context("Please include a fragment shader")?);

        // Create the actual pipeline
        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.pipeline_name),
                layout: Some(&layout),
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
                        format: self.texture_format,
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
                    vertex_buffers: &[
                        Vertex::desc(),
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false
            });

        Ok(pipeline)
    }
}