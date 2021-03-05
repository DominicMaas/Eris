mod c_body;
mod camera;
mod camera_controller;
mod mesh;
mod render_pipeline;
mod state;
mod texture;
mod uniform_buffer;
mod utils;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use futures::executor::block_on;
use std::time::Instant;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();

    // Create a window to use
    let window = WindowBuilder::new()
        .with_title("Eris Simulator")
        .build(&event_loop)
        .unwrap();

    // Setup ImGUI and attach it to our window, ImGui is used as the GUI for this
    // application
    //let mut imgui = imgui::Context::create();
    //let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    //platform.attach_window(
    //    imgui.io_mut(),
    //    &window,
    //    imgui_winit_support::HiDpiMode::Default,
    //);
    //imgui.set_ini_filename(None);

    // Setup the font for ImGui
    //let hidpi_factor = window.scale_factor();
    //let font_size = (13.0 * hidpi_factor) as f32;
    //imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    //imgui.fonts().add_font(&[FontSource::DefaultFontData {
    //    config: Some(imgui::FontConfig {
    //        oversample_h: 1,
    //        pixel_snap_h: true,
    ////        size_pixels: font_size,
    //        ..Default::default()
    //    }),
    //}]);

    //let renderer_config = RendererConfig {
    //    texture_format: display.sc_desc.format,
    //    ..Default::default()
    //};
    //let renderer = Renderer::new(&mut imgui, &display.device, &display.queue, renderer_config);

    let mut state = block_on(state::State::new(&window));
    let mut last_update = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = now - last_update;
                last_update = now;

                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }

                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
}
