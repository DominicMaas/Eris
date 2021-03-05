use winit::{event::*, window::Window};

use crate::c_body::CBody;
use crate::camera::Camera;
use crate::camera_controller::CameraController;
use crate::mesh::DrawMesh;
use crate::texture::Texture;
use crate::{render_pipeline, texture, uniform_buffer};
use cgmath::{InnerSpace, Vector3};
use imgui::FontSource;
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
    camera: Camera,
    camera_controller: CameraController,
    bodies: Vec<CBody>,
    gui_context: imgui::Context,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // The main camera
        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 150.0, 150.0).into(),
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
                    &Texture::create_bind_group_layout(&device),
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

        let camera_controller = CameraController::new(10.0);

        let mut bodies = Vec::new();

        let sun_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("images/sun.png"),
            "sun.png",
        )
        .unwrap();

        let sun = CBody::new(
            0,
            10000000.0,
            30.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            sun_texture,
            &device,
        );

        print!("Sun Escape Velocity: {}m/s", sun.escape_velocity());

        let inner_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("images/earth.png"),
            "earth.png",
        )
        .unwrap();

        let outer_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("images/earth.png"),
            "earth.png",
        )
        .unwrap();

        let inner_planet = CBody::new(
            1,
            100.0,
            6.0,
            cgmath::Vector3::new(60.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, -1.2),
            inner_texture,
            &device,
        );

        let outer_planet = CBody::new(
            1,
            100.0,
            6.0,
            cgmath::Vector3::new(160.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, -0.7),
            outer_texture,
            &device,
        );

        /*let c_body_earth = CBody::new(
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
        );*/

        bodies.push(sun);
        bodies.push(inner_planet);
        bodies.push(outer_planet);

        // -------------- GUI ------------------ //

        // Setup ImGUI and attach it to our window, ImGui is used as the GUI for this
        // application
        let mut gui_context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut gui_context);
        platform.attach_window(
            gui_context.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        gui_context.set_ini_filename(None);

        // Setup the font for ImGui
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        gui_context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        gui_context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: sc_desc.format,
            ..Default::default()
        };

        let gui_renderer =
            imgui_wgpu::Renderer::new(&mut gui_context, &device, &queue, renderer_config);

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
            camera,
            camera_controller,
            bodies,
            gui_context,
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
        // UI input
        self.gui_context.io_mut().update_delta_time(dt);

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

                body.velocity += acceleration * dt.as_secs_f32();
            }

            // Run simulations
            body.update();

            self.queue.write_buffer(
                &body.uniform_buffer.buffer,
                0,
                bytemuck::cast_slice(&[body.uniform_buffer.data]),
            );
        }

        // Update camera positions
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
                label: None,
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

            // ---- UI ---- //


            // Render bodies
            render_pass.set_pipeline(&self.c_body_pipeline);
            render_pass.set_bind_group(1, &self.uniform_buffer.bind_group, &[]);

            for body in self.bodies.iter() {
                render_pass.set_bind_group(0, &body.texture.bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(2, &body.uniform_buffer.bind_group, &[]);
                render_pass.draw_mesh(&body.mesh);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
