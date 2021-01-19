#version 450

layout(location = 0) in vec4 glPosition;
layout(location = 1) in vec4 point_position;
layout(location = 0) out vec4 pos_out;

void main() {
    gl_Position = glPosition;
    pos_out = point_position;
}
