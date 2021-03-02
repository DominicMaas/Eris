use cgmath::SquareMatrix;

pub struct Camera {

}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        //let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);
        // 2.
        //let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        //return OPENGL_TO_WGPU_MATRIX * proj * view;

        return cgmath::Matrix4::identity();
    }
}