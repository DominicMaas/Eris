use crate::utils::Vertex;
use wgpu::util::DeviceExt;

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, device: &wgpu::Device) -> Self {
        // Create a vertex buffer using the vertices
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        // We need this for rendering
        let num_vertices = vertices.len() as u32;

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