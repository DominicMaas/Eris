use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraUniform {
    pub view_proj: cgmath::Matrix4<f32>, // 4x4 matrix
}

unsafe impl bytemuck::Zeroable for CameraUniform {}
unsafe impl bytemuck::Pod for CameraUniform {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelUniform {
    pub model: cgmath::Matrix4<f32>, // 4x4 matrix
}

unsafe impl bytemuck::Zeroable for ModelUniform {}
unsafe impl bytemuck::Pod for ModelUniform {}

// A holder for a uniform buffer, contains the data and raw buffer
pub struct UniformBuffer<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
    //noinspection RsBorrowChecker
    /// Crate a new uniform buffer to store data of type
    pub fn new(data: T, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
        });

        Self { data, buffer }
    }
}
