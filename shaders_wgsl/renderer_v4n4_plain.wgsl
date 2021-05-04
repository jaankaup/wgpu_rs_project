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

// Ligth/material properties.
let light_pos: vec3<f32> = vec3<f32>(3.0, 48.0, 3.0);
let light_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
let material_spec_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
let material_shininess: f32 = 75.0;
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

    
    var surface_color: vec3<f32> = vec3<f32>(0.2,0.2,0.8);

    var specular_component: vec3<f32> = specular_coeffient * material_spec_color * light_color;
    var ambient_component:  vec3<f32> = ambient_coeffience * light_color * surface_color.xyz;
    var diffuse_component:  vec3<f32> = diff_coeffient * light_color * surface_color.xyz;
    
    var distance_to_light: f32 = distance(in.pos.xyz, light_pos); 
    var attentuation: f32 = 1.0 / (1.0 + attentuation_factor * pow(distance_to_light,2.0));
    
    var final_color: vec4<f32> = vec4<f32>(ambient_component + attentuation * (diffuse_component + specular_component) , 1.0);

    return final_color;
}
