use crate::uniform_buffer::{CameraUniform, UniformBuffer};
use crate::utils::OPENGL_TO_WGPU_MATRIX;
use cgmath::num_traits::FloatConst;
use cgmath::{Angle, EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

/// Holds the current projection of the program, this needs to be updated
/// whenever the window size changes
pub struct Projection {
    pub aspect: f32,
    pub fov_y: Rad<f32>,
    pub z_near: f32,
    pub z_far: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fov_y: Rad<f32>, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y,
            z_near,
            z_far,
        }
    }

    /// When the window resizes, this updates the internal
    /// aspect ratio to ensure everything still looks correct
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculate the projection matrix for the window
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(self.fov_y, self.aspect, self.z_near, self.z_far)
    }
}

/// Holds the camera position, yaw and pitch
pub struct Camera {
    pub position: Vector3<f32>,

    pub front: Vector3<f32>,
    pub up: Vector3<f32>,
    pub world_up: Vector3<f32>,
    pub right: Vector3<f32>,

    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,

    pub projection: Projection,
    pub uniform_buffer: UniformBuffer<CameraUniform>,
}

impl Camera {
    pub fn new(position: Vector3<f32>, projection: Projection, device: &wgpu::Device) -> Self {
        // The uniform buffer
        let uniform_buffer = UniformBuffer::new(
            "Camera Uniform Buffer",
            wgpu::ShaderStage::VERTEX,
            CameraUniform {
                view_proj: Matrix4::identity(),
            },
            &device,
        );

        Self {
            position,
            front: (0.0, 0.0, 0.0).into(), // Where the camera is looking (takes into account rotation)
            up: (0.0, 0.0, 0.0).into(),
            world_up: (0.0, 1.0, 0.0).into(),
            right: (0.0, 0.0, 0.0).into(),
            yaw: cgmath::Rad(-90.0 / 180.0 * f32::PI()), // Look left or right
            pitch: cgmath::Rad(0.0),                     // Look Up / Down
            projection,
            uniform_buffer,
        }
    }

    /// Calculate the view matrix for the camera
    fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        Matrix4::look_at_rh(
            Point3::from_vec(self.position),
            Point3::from_vec(self.position + self.front),
            self.up,
        )
    }

    /// Update the uniforms for the camera, and write to the GPU
    pub fn update_uniforms(&mut self, queue: &wgpu::Queue) {
        self.uniform_buffer.data.view_proj = self.projection.calc_matrix() * self.calc_matrix();

        queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_buffer.data]),
        );
    }
}

pub struct CameraController {
    moving_left: bool,
    moving_right: bool,
    moving_forward: bool,
    moving_backward: bool,
    moving_up: bool,
    moving_down: bool,

    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            moving_left: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
            moving_up: false,
            moving_down: false,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.moving_up = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.moving_down = is_pressed;
                        true
                    }
                    VirtualKeyCode::W => {
                        self.moving_forward = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.moving_left = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.moving_backward = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.moving_right = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        let velocity = self.speed * dt;

        // Update Positions (left, right)
        if self.moving_left {
            camera.position -= camera.right * velocity;
        }

        if self.moving_right {
            camera.position += camera.right * velocity;
        }

        // Update positions (forward, backward)
        if self.moving_forward {
            camera.position += camera.front * velocity;
        }

        if self.moving_backward {
            camera.position -= camera.front * velocity;
        }

        // Update positions (up, down)
        if self.moving_up {
            camera.position += camera.up * velocity;
        }

        if self.moving_down {
            camera.position -= camera.up * velocity;
        }

        // Update mouse

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(FRAC_PI_2) {
            camera.pitch = -Rad(FRAC_PI_2);
        } else if camera.pitch > Rad(FRAC_PI_2) {
            camera.pitch = Rad(FRAC_PI_2);
        }

        // Update internals

        // Calculate the new Front vector
        camera.front = Vector3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        )
        .normalize();

        // Also re-calculate the Right and Up vector
        // Normalize the vectors, because their length gets closer
        // to 0 the more you look up or down which results in slower movement.
        camera.right = camera.front.cross(camera.world_up).normalize();
        camera.up = camera.right.cross(camera.front).normalize();
    }
}
