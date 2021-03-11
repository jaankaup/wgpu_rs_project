#version 450

layout(location = 0) flat in uint color_out;
//layout(location = 0) in uint color_out;

layout(location = 0) out vec4 final_color;

vec4 decode_color(uint c) {
  float a = (c & 0xff) / 255.0;     
  float b = ((c & 0xff00) >> 8) / 255.0;
  float g = ((c & 0xff0000) >> 16) / 255.0;    
  float r = ((c & 0xff000000) >> 24) / 255.0;    
  return vec4(r,g,b,a);
}

void main() {
    final_color = vec4(decode_color(color_out).xyz, 1.0);
}

