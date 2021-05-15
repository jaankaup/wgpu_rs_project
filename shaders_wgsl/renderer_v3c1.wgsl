struct VertexOutput {
    [[builtin(position)]] my_pos: vec4<f32>;
    [[location(0)]] pos: vec3<f32>;
    [[location(1)]] col: u32;
    //[[location(1), interpolate(flat)]] col: u32;
};

[[block]]
struct Camera {
    u_view_proj: mat4x4<f32>;
    camera_pos: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camerauniform: Camera;

// fn decode_color(c: u32) -> vec4<f32> {
//   let a: f32 = f32(c & 0xff) / 255.0;
//   let b: f32 = f32((c & 0xff00) >> 8) / 255.0;
//   let g: f32 = f32((c & 0xff0000) >> 16) / 255.0;
//   let r: f32 = f32((c & 0xff000000) >> 24) / 255.0;
//   return vec4<f32>(r,g,b,a);
// }

[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec3<f32>, [[location(1)]] col: u32) -> VertexOutput {
    var out: VertexOutput;
    out.my_pos = camerauniform.u_view_proj * vec4<f32>(pos , 1.0);
    out.pos = pos;
    out.col = col;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0,2.0,3.0,4.0);
    //return vec4<f32>(decode_color(in.col).xyz, 1.0);
}