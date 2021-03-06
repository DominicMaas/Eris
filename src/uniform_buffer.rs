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

// main.rs
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightUniform {
    pub position: cgmath::Vector3<f32>,
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for LightUniform {}
unsafe impl bytemuck::Pod for LightUniform {}

impl LightUniform {
    pub fn new(position: cgmath::Vector3<f32>, color: cgmath::Vector3<f32>) -> Self {
        Self {
            position,
            _padding: 0,
            color,
        }
    }
}

// A holder for a uniform buffer, contains the data and raw buffer
pub struct UniformBuffer<T>
where
    T: Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub data: T,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
    //noinspection RsBorrowChecker
    /// Crate a new uniform buffer to store data of type
    pub fn new(name: &str, visibility: wgpu::ShaderStage, data: T, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(name),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &UniformBufferUtils::create_bind_group_layout(visibility, &device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        Self {
            data,
            buffer,
            bind_group,
        }
    }
}

pub struct UniformBufferUtils {}
impl UniformBufferUtils {
    pub fn create_bind_group_layout(
        visibility: wgpu::ShaderStage,
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        bind_group_layout
    }
}
