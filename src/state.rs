use winit::{event::*, window::Window};

use crate::c_body::CBody;
use crate::camera::Camera;
use crate::camera_controller::CameraController;
use crate::mesh::{DrawMesh, Mesh};
use crate::uniform_buffer::UniformBuffer;
use crate::utils::{Vertex, G, SIM_SCALE, SIM_SPEED};
use crate::{render_pipeline, texture, uniform_buffer};
use cgmath::num_traits::Pow;
use cgmath::{EuclideanSpace, InnerSpace, MetricSpace, Point3, Vector3};
use std::time::Duration;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    c_body_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    uniform_buffer: uniform_buffer::UniformBuffer<uniform_buffer::CameraUniform>,
    mesh_uniform_buffer: uniform_buffer::UniformBuffer<uniform_buffer::ModelUniform>,
    mesh: Mesh,
    diffuse_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    bodies: Vec<CBody>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // The main camera
        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 50000.0, 1.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 70.0,
            znear: 0.1,
            zfar: 1000000.0,
        };

        // The uniform buffer
        let uniform_buffer = uniform_buffer::UniformBuffer::new(
            uniform_buffer::CameraUniform {
                view_proj: camera.build_view_projection_matrix(),
            },
            &device,
        );

        // Pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_buffer::UniformBufferUtils::create_bind_group_layout(&device),
                    &uniform_buffer::UniformBufferUtils::create_bind_group_layout(&device),
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            render_pipeline::RenderPipelineBuilder::new(sc_desc.format, "Main Pipeline")
                .with_vertex_shader(wgpu::include_spirv!("shaders/shader.vert.spv"))
                .with_fragment_shader(wgpu::include_spirv!("shaders/shader.frag.spv"))
                .with_layout(&render_pipeline_layout)
                .build(&device)
                .unwrap();

        let c_body_pipeline =
            render_pipeline::RenderPipelineBuilder::new(sc_desc.format, "C Body Pipeline")
                .with_vertex_shader(wgpu::include_spirv!("shaders/c_body_shader.vert.spv"))
                .with_fragment_shader(wgpu::include_spirv!("shaders/c_body_shader.frag.spv"))
                .with_layout(&render_pipeline_layout)
                .with_topology(wgpu::PrimitiveTopology::LineList)
                .build(&device)
                .unwrap();

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        // The mesh
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        //vertices.push(Vertex::with_color(cgmath::Vector3::new(-0.0868241, 0.49240386, 0.0), cgmath::Vector3::new(0.5, 0.0, 0.5)));
        //vertices.push(Vertex::with_color(cgmath::Vector3::new(-0.49513406, 0.06958647, 0.0), cgmath::Vector3::new(0.5, 0.0, 0.5)));
        //vertices.push(Vertex::with_color(cgmath::Vector3::new(-0.21918549, -0.44939706, 0.0), cgmath::Vector3::new(0.5, 0.0, 0.5)));
        //vertices.push(Vertex::with_color(cgmath::Vector3::new(0.35966998, -0.3473291, 0.0), cgmath::Vector3::new(0.5, 0.0, 0.5)));
        //vertices.push(Vertex::with_color(cgmath::Vector3::new(0.44147372, 0.2347359, 0.0), cgmath::Vector3::new(0.5, 0.0, 0.5)));

        vertices.push(Vertex::with_tex_coords(
            cgmath::Vector3::new(-0.0868241, 0.49240386, 0.0),
            cgmath::Vector3::new(1.0, 1.0, 1.0),
            cgmath::Vector2::new(0.4131759, 0.99240386),
        ));
        vertices.push(Vertex::with_tex_coords(
            cgmath::Vector3::new(-0.49513406, 0.06958647, 0.0),
            cgmath::Vector3::new(1.0, 1.0, 1.0),
            cgmath::Vector2::new(0.0048659444, 0.56958646),
        ));
        vertices.push(Vertex::with_tex_coords(
            cgmath::Vector3::new(-0.21918549, -0.44939706, 0.0),
            cgmath::Vector3::new(1.0, 1.0, 1.0),
            cgmath::Vector2::new(0.28081453, 0.050602943),
        ));
        vertices.push(Vertex::with_tex_coords(
            cgmath::Vector3::new(0.35966998, -0.3473291, 0.0),
            cgmath::Vector3::new(1.0, 1.0, 1.0),
            cgmath::Vector2::new(0.85967, 0.15267089),
        ));
        vertices.push(Vertex::with_tex_coords(
            cgmath::Vector3::new(0.44147372, 0.2347359, 0.0),
            cgmath::Vector3::new(1.0, 1.0, 1.0),
            cgmath::Vector2::new(0.9414737, 0.7347359),
        ));

        indices.push(0);
        indices.push(1);
        indices.push(4);

        indices.push(1);
        indices.push(2);
        indices.push(4);

        indices.push(2);
        indices.push(3);
        indices.push(4);

        let mesh = Mesh::new(vertices, indices, &device);
        let mesh_uniform_buffer = uniform_buffer::UniformBuffer::new(
            uniform_buffer::ModelUniform {
                model: cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0)),
            },
            &device,
        );

        // The texture (temp)
        let diffuse_bytes = include_bytes!("images/happy-tree.png"); // CHANGED!
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap(); // CHANGED!

        // ----- BIND GROUPS ----- //
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_controller = CameraController::new(60.0);

        let mut bodies = Vec::new();

        let c_body_earth = CBody::new(
            0,
            5.972e24 * SIM_SCALE,
            6.371e6 * SIM_SCALE,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            &device,
        );

        let c_body_moon = CBody::new(
            1,
            7.342e22 * SIM_SCALE,
            1.7371e6 * SIM_SCALE,
            cgmath::Vector3::new(384.4e6 * SIM_SCALE, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, -4022.0 * SIM_SCALE * SIM_SPEED),
            &device,
        );

        bodies.push(c_body_earth);
        bodies.push(c_body_moon);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            c_body_pipeline,
            depth_texture,
            uniform_buffer,
            mesh,
            mesh_uniform_buffer,
            diffuse_bind_group,
            camera,
            camera_controller,
            bodies,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        // After the swapchain is recreated, we need to rebuild the depth texture
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");

        // The camera sizing needs to be updated
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self, dt: Duration) {
        // Loop through all bodies and apply updates
        for i in 0..self.bodies.len() {
            let (before, nonbefore) = self.bodies.split_at_mut(i);
            let (body, after) = nonbefore.split_first_mut().unwrap();

            // Calculate net force against other bodies

            // This loop iterates over all bodies that are no the current body
            for body2 in before.iter().chain(after.iter()) {
                let sqr_distance: f32 = (body2.position - body.position).magnitude2();
                let force_direction: Vector3<f32> = (body2.position - body.position).normalize();
                let force: Vector3<f32> =
                    force_direction * body.standard_gravitational_parameter() * body2.mass
                        / sqr_distance;
                let acceleration: Vector3<f32> = force / body.mass;

                body.velocity += acceleration;
            }

            // Run simulations
            //print!("Body {} V(x,y,z): V({},{},{})\n", body.id, body.velocity.x, body.velocity.y, body.velocity.z);
            body.update();

            self.queue.write_buffer(
                &body.uniform_buffer.buffer,
                0,
                bytemuck::cast_slice(&[body.uniform_buffer.data]),
            );
        }

        // Look towards main mass
        //self.camera.target = Point3::from_vec(self.bodies[0].position);

        self.camera_controller.update_camera(&mut self.camera);
        self.uniform_buffer.data.view_proj = self.camera.build_view_projection_matrix();
        self.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_buffer.data]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // Get a frame
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_buffer.bind_group, &[]);
            render_pass.set_bind_group(2, &self.mesh_uniform_buffer.bind_group, &[]);

            render_pass.draw_mesh(&self.mesh);

            // Render bodies
            render_pass.set_pipeline(&self.c_body_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_buffer.bind_group, &[]);

            for body in self.bodies.iter() {
                render_pass.set_bind_group(2, &body.uniform_buffer.bind_group, &[]);
                render_pass.draw_mesh(&body.mesh);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
