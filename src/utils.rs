#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// The scale of this simulation, stuff will get messy if we scale realtime
/// This gets applied to all real life calculations
pub const SIM_SCALE: f32 = 0.0001;

/// The speed of this simulation, by default it is realtime, which does not help at all
pub const SIM_SPEED: f32 = 800.0;

pub const G: f32 = 6.67e-11f32 * SIM_SCALE;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
    pub tex_coord: cgmath::Vector2<f32>,
    pub normal: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    /// Create a vertex with color
    #[allow(dead_code)]
    pub fn with_color(position: cgmath::Vector3<f32>, color: cgmath::Vector3<f32>) -> Self {
        Vertex {
            position,
            color,
            tex_coord: cgmath::Vector2::new(0.0, 0.0),
            normal: cgmath::Vector3::new(0.0, 0.0, 0.0),
        }
    }

    /// Create a vertex with tex coords
    pub fn with_tex_coords(
        position: cgmath::Vector3<f32>,
        normal: cgmath::Vector3<f32>,
        tex_coord: cgmath::Vector2<f32>,
    ) -> Self {
        Vertex {
            position,
            color: cgmath::Vector3::new(0.0, 0.0, 0.0),
            tex_coord,
            normal,
        }
    }

    pub(crate) fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<cgmath::Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<cgmath::Vector3<f32>>() * 2)
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<cgmath::Vector3<f32>>() * 2
                        + std::mem::size_of::<cgmath::Vector2<f32>>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}
