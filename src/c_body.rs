use cgmath::prelude::*;
use cgmath::Vector3;

pub struct CBody {
    pub mass: f32,
    pub radius: f32,
    pub velocity: Vector3<f32>,
}

impl CBody {
    pub fn new(mass: f32, radius: f32, velocity: Vector3<f32>) -> Self {
        Self {
            mass,
            radius,
            velocity
        }
    }
}