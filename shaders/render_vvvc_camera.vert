#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in uint col;

layout(location = 0) flat out uint color_out;

layout(set=0, binding=0) uniform camerauniform {
    mat4 u_view_proj;
    vec4 camera_pos;
};

void main() {
    gl_PointSize = 4.0;
    gl_Position = u_view_proj * vec4(pos.xyz, 1.0); 
    color_out = col;
}
