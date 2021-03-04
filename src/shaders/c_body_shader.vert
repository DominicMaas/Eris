// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;
layout(location=2) in vec2 a_tex_coords;

layout(location=0) out vec3 v_color;
layout(location=1) out vec2 v_tex_coords;

layout(set=1, binding=0) uniform CameraUniforms {
    mat4 u_view_proj;
};

layout(set=2, binding=0) uniform ModelUniforms {
    mat4 u_model;
};

void main() {
    v_color = a_color;
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * u_model * vec4(a_position, 1.0);
}