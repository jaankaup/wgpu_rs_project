#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in uint col;

layout(location = 0) flat out uint color_out;

layout(set=0, binding=0) uniform camerauniform {
    mat4 u_view_proj;
    vec4 camera_pos;
};

vec4 decode_color(uint c) {
  float a = (c & 0xff) / 255.0;
  float b = ((c & 0xff00) >> 8) / 255.0;
  float g = ((c & 0xff0000) >> 16) / 255.0;
  float r = ((c & 0xff000000) >> 24) / 255.0;
  return vec4(r,g,b,a);
}

void main() {
    gl_PointSize = min(2.0, decode_color(col).w / 255.0);
    gl_Position = u_view_proj * vec4(pos.xyz, 1.0);
    color_out = col;
}
