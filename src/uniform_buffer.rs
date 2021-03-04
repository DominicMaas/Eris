use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
// The actual data to be stored in the GPU
struct UniformData {
    view_proj: cgmath::Matrix4<f32> // 4x4 matrix
}

unsafe impl bytemuck::Zeroable for UniformData {}
unsafe impl bytemuck::Pod for UniformData {}

// A holder for a uniform buffer, contains the data and raw buffer
pub struct UniformBuffer {
    data: UniformData,
    buffer: wgpu::Buffer
}

impl UniformBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        use cgmath::SquareMatrix;

        let view_proj = cgmath::Matrix4::identity();
        let mut data = UniformData { view_proj };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents:  bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM
        });

        Self { data, buffer }
    }
}
