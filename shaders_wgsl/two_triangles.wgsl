struct VertexOutput {
    [[builtin(position)]] my_pos: vec4<f32>;
    [[location(0)]] pos_out: vec4<f32>;
};

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;

[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(vertex)]]
fn vs_main([[location(0)]] gl_pos: vec4<f32>, [[location(1)]] point_pos: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.my_pos = gl_pos;
    out.pos_out = point_pos;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.pos_out.xy);
}
