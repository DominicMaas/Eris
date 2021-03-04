use cgmath::{Vector3, Quaternion};
use crate::mesh::Mesh;
use crate::utils::Vertex;
use cgmath::num_traits::FloatConst;
use crate::uniform_buffer::{UniformBuffer, ModelUniform};

pub struct CBody {
    pub mass: f32,
    pub radius: f32,
    pub velocity: Vector3<f32>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub mesh: Mesh,
    pub uniform_buffer: UniformBuffer<ModelUniform>,
}

impl CBody {
    pub fn new(mass: f32, radius: f32, velocity: Vector3<f32>, device: &wgpu::Device) -> Self {
        // Create the mesh for this body
        let sector_count: u16 = 38;
        let stack_count: u16 = 24;
        let mesh = Self::build_mesh(radius, sector_count, stack_count, device);
        let position: Vector3<f32> = Vector3::new(0.0,0.0,0.0);
        let rotation: Quaternion<f32> = Quaternion::new(0.0, 0.0, 0.0, 0.0);

        let uniform_data = ModelUniform {
            model: cgmath::Matrix4::from_translation(position) * cgmath::Matrix4::from(rotation)
        };

        let uniform_buffer = UniformBuffer::new(uniform_data, device);

        Self {
            mass,
            radius,
            velocity,
            position,
            rotation,
            mesh,
            uniform_buffer
        }
    }


    fn build_mesh(radius: f32, sector_count: u16, stack_count: u16, device: &wgpu::Device) -> Mesh {
        // Build the vertices for the mesh
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        // vertex position
        let mut x: f32;
        let mut y: f32;
        let mut z: f32;
        let mut xy: f32;

        // vertex normal
        let mut nx: f32;
        let mut ny: f32;
        let mut nz: f32;
        let length_inv: f32 = 1.0 / radius;

        // vertex texCoord
        let mut s: f32;
        let mut t: f32;

        let sector_step: f32 = 2.0 * f32::PI() / sector_count as f32;
        let stack_step: f32 = f32::PI() / stack_count as f32;
        let mut sector_angle: f32;
        let mut stack_angle: f32;

        for i in 0..stack_count+1 {
            stack_angle = f32::PI() / 2.0 - i as f32 * stack_step;  // starting from pi/2 to -pi/2
            xy = radius * stack_angle.cos();               // r * cos(u)
            z = radius * stack_angle.sin();                // r * sin(u)

            // add (sectorCount+1) vertices per stack
            // the first and last vertices have same position and normal, but different tex coords
            for j in 0..sector_count+1 {
                sector_angle = j as f32 * sector_step;              // starting from 0 to 2pi

                // vertex position (x, y, z)
                x = xy * sector_angle.cos();             // r * cos(u) * cos(v)
                y = xy * sector_angle.sin();             // r * cos(u) * sin(v)

                let position = cgmath::Vector3::new(x, y, z);

                // normalized vertex normal (nx, ny, nz)
                nx = x * length_inv;
                ny = y * length_inv;
                nz = z * length_inv;

                let normal = cgmath::Vector3::new(nx, ny, nz);

                // vertex tex coord (s, t) range between [0, 1]
                s = (j / sector_count) as f32;
                t = (i / stack_count) as f32;

                let tex_coord = cgmath::Vector2::new(s, t,);

                vertices.push(Vertex::with_tex_coords(position, normal, tex_coord));
            }
        }

        let mut k1: u16;
        let mut k2: u16;

        for i in 0u16..stack_count {
            k1 = i * (sector_count + 1);     // beginning of current stack
            k2 = k1 + sector_count + 1;      // beginning of next stack

            for _j in 0u16..sector_count {

                // 2 triangles per sector excluding first and last stacks
                // k1 => k2 => k1+1
                if i != 0 {
                    indices.push(k1);
                    indices.push(k2);
                    indices.push(k1 + 1);
                }

                // k1+1 => k2 => k2+1
                if i != (stack_count - 1) {
                    indices.push(k1 + 1);
                    indices.push(k2);
                    indices.push(k2 + 1);
                }

                k1 += 1;
                k2 += 1;
            }
        }

        // Create the mesh for this body
        Mesh::new(vertices, indices, device)
    }
}