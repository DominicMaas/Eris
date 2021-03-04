use crate::utils::Vertex;
use wgpu::util::DeviceExt;

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32
}

impl Mesh {
    pub fn new(device: &wgpu::Device) -> Self {
        // Temp, this will be passed in as a type of vector or something
        const VERTICES: &[Vertex] = &[
            Vertex { position: cgmath::Vector3::new(0.0, 0.5, 0.0), color: cgmath::Vector3::new(1.0, 0.0, 0.0) },
            Vertex { position: cgmath::Vector3::new(-0.5, -0.5, 0.0), color: cgmath::Vector3::new(0.0, 1.0, 0.0) },
            Vertex { position: cgmath::Vector3::new(0.5, -0.5, 0.0), color: cgmath::Vector3::new(0.0, 0.0, 1.0) },
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let num_vertices = VERTICES.len() as u32;

        Self {
            vertex_buffer,
            num_vertices
        }
    }
}

pub trait DrawMesh<'a, 'b>
    where 'b: 'a, {
    fn draw_mesh(&mut self, mesh: &'b Mesh);
}

impl<'a, 'b> DrawMesh<'a, 'b> for wgpu::RenderPass<'a>
    where 'b: 'a, {
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.draw(0..mesh.num_vertices, 0..1);
    }
}