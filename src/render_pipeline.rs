use crate::texture;
use crate::utils::Vertex;
use anyhow::*;

pub struct RenderPipelineBuilder<'a> {
    layout: Option<&'a wgpu::PipelineLayout>,
    vertex_shader_source: Option<wgpu::ShaderModuleDescriptor<'a>>,
    fragment_shader_source: Option<wgpu::ShaderModuleDescriptor<'a>>,
    texture_format: wgpu::TextureFormat,
    pipeline_name: &'a str,
    primitive_topology: wgpu::PrimitiveTopology,
}
impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        texture_format: wgpu::TextureFormat,
        pipeline_name: &'a str,
    ) -> RenderPipelineBuilder {
        Self {
            layout: None,
            vertex_shader_source: None,
            fragment_shader_source: None,
            texture_format,
            pipeline_name,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        }
    }

    pub fn with_layout(&mut self, layout: &'a wgpu::PipelineLayout) -> &mut Self {
        self.layout = Some(layout);
        self
    }

    pub fn with_vertex_shader(
        &mut self,
        vertex_shader: wgpu::ShaderModuleDescriptor<'a>,
    ) -> &mut Self {
        self.vertex_shader_source = Some(vertex_shader);
        self
    }

    pub fn with_fragment_shader(
        &mut self,
        fragment_shader: wgpu::ShaderModuleDescriptor<'a>,
    ) -> &mut Self {
        self.fragment_shader_source = Some(fragment_shader);
        self
    }

    pub fn with_topology(&mut self, topology: wgpu::PrimitiveTopology) -> &mut Self {
        self.primitive_topology = topology;
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
        let vs_module = device.create_shader_module(
            &self
                .vertex_shader_source
                .take()
                .context("Please include a vertex shader")?,
        );
        let fs_module = device.create_shader_module(
            &self
                .fragment_shader_source
                .take()
                .context("Please include a fragment shader")?,
        );

        // Create the actual pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.pipeline_name),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: self.primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: self.texture_format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        /* let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),


        });*/

        Ok(pipeline)
    }
}
