#version 450

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 nor;

layout(location = 0) out vec4 pos_f;
layout(location = 1) out vec4 nor_f;

layout(set=0, binding=0) uniform camerauniform {
    mat4 u_view_proj;
    vec4 camera_pos;
};

void main() {
    gl_Position = u_view_proj * vec4(pos.xyz, 1.0); 
    pos_f = pos;
    nor_f = nor; 
}
