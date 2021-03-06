use winit::{event::*, window::Window};

use crate::c_body::CBody;
use crate::mesh::DrawMesh;
use crate::texture::Texture;
use crate::{camera, render_pipeline, texture, uniform_buffer};
use cgmath::num_traits::FloatConst;
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
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_projection: camera::Projection,
    bodies: Vec<CBody>,
    gui_context: imgui::Context,
    gui_platform: imgui_winit_support::WinitPlatform,
    gui_renderer: imgui_wgpu::Renderer,
    mouse_pressed: bool,
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
        let camera_projection = camera::Projection::new(
            sc_desc.width,
            sc_desc.height,
            cgmath::Rad(70.0 / 180.0 * f32::PI()),
            0.01,
            1000.0,
        );
        let camera_controller = camera::CameraController::new(20.0, 0.1);
        let camera = camera::Camera {
            position: (0.0, 0.0, 0.0).into(), // Position of camera
            front: (0.0, 0.0, 0.0).into(), // Where the camera is looking (takes into account rotation)
            up: (0.0, 0.0, 0.0).into(),
            world_up: (0.0, 1.0, 0.0).into(),
            right: (0.0, 0.0, 0.0).into(),
            yaw: cgmath::Rad(-90.0 / 180.0 * f32::PI()), // Look left or right
            pitch: cgmath::Rad(0.0),                     // Look Up / Down
        };

        // The uniform buffer
        let uniform_buffer = uniform_buffer::UniformBuffer::new(
            uniform_buffer::CameraUniform {
                view_proj: camera_projection.calc_matrix() * camera.calc_matrix(),
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
            10000.0,
            32.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            sun_texture,
            &device,
        );

        print!("Sun Escape Velocity: {}m/s\n", sun.escape_velocity());

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
            10.0,
            12.0,
            cgmath::Vector3::new(64.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, -0.02795084),
            inner_texture,
            &device,
        );

        print!("Inner Planet Initial Velocity: {}m/s\n", inner_planet.velocity.magnitude());

        let outer_planet = CBody::new(
            1,
            2.0,
            4.0,
            cgmath::Vector3::new(-128.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.01976423),
            outer_texture,
            &device,
        );

        bodies.push(sun);
        bodies.push(inner_planet);
        bodies.push(outer_planet);

        // -------------- GUI ------------------ //

        // Setup ImGUI and attach it to our window, ImGui is used as the GUI for this
        // application
        let mut gui_context = imgui::Context::create();
        let mut gui_platform = imgui_winit_support::WinitPlatform::init(&mut gui_context);
        gui_platform.attach_window(
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
            camera_projection,
            bodies,
            gui_context,
            gui_platform,
            gui_renderer,
            mouse_pressed: false,
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

        // The screen projection needs to be updated
        self.camera_projection
            .resize(self.sc_desc.width, self.sc_desc.height);
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                //if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                //}
                true
            }
            _ => false,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_keyboard(event)
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

                body.velocity += acceleration;
            }

            // Run simulations
            body.update(dt);

            self.queue.write_buffer(
                &body.uniform_buffer.buffer,
                0,
                bytemuck::cast_slice(&[body.uniform_buffer.data]),
            );
        }

        // Update camera positions
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniform_buffer.data.view_proj =
            self.camera_projection.calc_matrix() * self.camera.calc_matrix();
        self.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_buffer.data]),
        );
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SwapChainError> {
        // Build the UI
        self.gui_platform
            .prepare_frame(self.gui_context.io_mut(), &window)
            .expect("Failed to prepare frame!");

        let ui = self.gui_context.frame();
        {
            let window = imgui::Window::new(imgui::im_str!("Debug"));
            window
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                });
        }

        // Get a frame
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(&Default::default());

        // ---- Main ---- //
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
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

            // Render bodies
            render_pass.set_pipeline(&self.c_body_pipeline);
            render_pass.set_bind_group(1, &self.uniform_buffer.bind_group, &[]);

            for body in self.bodies.iter() {
                render_pass.set_bind_group(0, &body.texture.bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(2, &body.uniform_buffer.bind_group, &[]);
                render_pass.draw_mesh(&body.mesh);
            }
        }

        // ---- UI ---- //
        {
            let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            // Render the UI
            self.gui_platform.prepare_render(&ui, &window);
            self.gui_renderer
                .render(ui.render(), &self.queue, &self.device, &mut ui_pass)
                .expect("Failed to render UI!");
        }

        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
