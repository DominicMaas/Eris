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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Handle guid events
        let io = state.gui_context.io_mut();
        state.gui_platform.handle_event(io, &window, &event);

        match event {
            Event::RedrawRequested(_) => {
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
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::DeviceEvent { ref event, .. } => {
                state.device_input(event);
            }
            Event::WindowEvent { ref event, .. } => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
