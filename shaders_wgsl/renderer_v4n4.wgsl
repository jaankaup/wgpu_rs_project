struct VertexOutput {
    [[builtin(position)]] my_pos: vec4<f32>;
    [[location(0)]] pos: vec4<f32>;
    [[location(1)]] nor: vec4<f32>;
};

[[block]]
struct Camera {
    u_view_proj: mat4x4<f32>;
    camera_pos: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camerauniform: Camera;

[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec4<f32>, [[location(1)]] nor: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.my_pos = camerauniform.u_view_proj * pos;
    out.pos = pos;
    out.nor = nor;
    return out;
}

// Textures.

[[group(1), binding(0)]]
var t_diffuse1: texture_2d<f32>;

[[group(1), binding(1)]]
var s_diffuse1: sampler;

[[group(1), binding(2)]]
var t_diffuse2: texture_2d<f32>;

[[group(1), binding(3)]]
var s_diffuse2: sampler;

let light_pos: vec3<f32> = vec3<f32>(3.0, 48.0, 3.0);
let light_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
let material_spec_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
let material_shininess: f32 = 70.0;
let ambient_coeffience: f32 = 0.15;
let attentuation_factor: f32 = 0.009;

[[stage(fragment)]]

fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    var light_dir: vec3<f32> = normalize(light_pos - in.pos.xyz);
    var normal: vec3<f32> = normalize(in.nor).xyz; // is this necessery? 
    var diff_coeffient: f32 = max(0.0, dot(normal, light_dir));
    var reflection_vector: vec3<f32> = reflect(-light_dir, normal);
    var camera_dir: vec3<f32> = normalize(camerauniform.camera_pos.xyz - in.pos.xyz);
    
    var cosAngle: f32 = max(0.0, dot(camera_dir, reflection_vector));
    var specular_coeffient: f32 = 0.0;

    if (diff_coeffient > 0.0) {
        specular_coeffient = pow(cosAngle, material_shininess);
    }

    var offset_factor: f32 = 1.5;
    
    var coord1: vec2<f32> = in.pos.xy*offset_factor;
    var coord2: vec2<f32> = in.pos.xz*offset_factor;
    var coord3: vec2<f32> = in.pos.zz*offset_factor;
    
    var surfaceColor_grass: vec3<f32> = textureSample(t_diffuse1, s_diffuse1, offset_factor * (coord1 + coord2 + coord3) / 3.0).xyz; //, offset_factor * (coord1 + coord2 + coord3) / 3.0).xyz;
    var surfaceColor_rock:  vec3<f32>  = textureSample(t_diffuse2, s_diffuse2, 1.1 * (coord1 + coord2 + coord3) / 3.0).xyz;
    var surface_color: vec3<f32> = mix(
        surfaceColor_rock, surfaceColor_grass,
        vec3<f32>(clamp(0.4*in.nor.x + 0.6*in.nor.y, 0.0, 1.0)));

    var specular_component: vec3<f32> = specular_coeffient * material_spec_color * light_color;
    var ambient_component:  vec3<f32> = ambient_coeffience * light_color * surface_color.xyz;
    var diffuse_component:  vec3<f32> = diff_coeffient * light_color * surface_color.xyz;
    
    var distance_to_light: f32 = distance(in.pos.xyz, light_pos); 
    var attentuation: f32 = 1.0 / (1.0 + attentuation_factor * pow(distance_to_light,2.0));
    
    var final_color: vec4<f32> = vec4<f32>(ambient_component + attentuation * (diffuse_component + specular_component) , 1.0);

    return final_color;
}