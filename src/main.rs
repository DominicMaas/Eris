mod c_body;
mod camera;
mod mesh;
mod render_pipeline;
mod state;
mod texture;
mod uniform_buffer;
mod utils;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
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

    let mut state = block_on(state::State::new(&window));
    let mut last_update = Instant::now();
    let mut is_resumed = true;
    let mut is_focused = true;
    let mut is_redraw_requested = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if is_resumed && is_focused {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };

        match event {
            Event::Resumed => is_resumed = true,
            Event::Suspended => is_resumed = false,
            Event::RedrawRequested(wid) => {
                if wid == window.id() {
                    let now = Instant::now();
                    let dt = now - last_update;
                    last_update = now;

                    state.update(dt);

                    match state.render(&window) {
                        Ok(_) => {}
                        // Recreate the swap_chain if lost
                        Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }

                    is_redraw_requested = false;
                }
            }

            Event::MainEventsCleared => {
                if is_focused && is_resumed && !is_redraw_requested {
                    window.request_redraw();
                    is_redraw_requested = true;
                } else {
                    // Freeze time while the demo is not in the foreground
                    last_update = Instant::now();
                }
            }

            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                state.device_input(event);
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Focused(f) => is_focused = *f,

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
