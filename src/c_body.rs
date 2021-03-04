use cgmath::prelude::*;
use cgmath::Vector3;
use crate::mesh::Mesh;
use crate::utils::Vertex;
use cgmath::num_traits::FloatConst;

pub struct CBody {
    pub mass: f32,
    pub radius: f32,
    pub velocity: Vector3<f32>,
    pub mesh: Mesh,
}

impl CBody {
    pub fn new(mass: f32, radius: f32, velocity: Vector3<f32>, device: &wgpu::Device) -> Self {
        // Create the mesh for this body
        let mesh = Self::build_mesh(device);

        Self {
            mass,
            radius,
            velocity,
            mesh
        }
    }


    fn build_mesh(device: &wgpu::Device) -> Mesh {
        let radius: f32 = 1.0;
        let sector_count: u16 = 38;
        let stack_count: u16 = 24;

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
        let mut length_inv: f32 = 1.0 / radius;

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

                // normalized vertex normal (nx, ny, nz)
                nx = x * length_inv;
                ny = y * length_inv;
                nz = z * length_inv;

                // vertex tex coord (s, t) range between [0, 1]
                s = (j / sector_count) as f32;
                t = (i / stack_count) as f32;

                vertices.push(Vertex::with_tex_coords(cgmath::Vector3::new(x, y, z), cgmath::Vector2::new(s, t,)));
            }
        }

        let mut k1: u16;
        let mut k2: u16;

        for i in 0u16..stack_count {
            k1 = i * (sector_count + 1);     // beginning of current stack
            k2 = k1 + sector_count + 1;      // beginning of next stack

            for j in 0u16..sector_count {

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

        // indices.clear();

        // Create the mesh for this body
        Mesh::new(vertices, indices, device)
    }
}