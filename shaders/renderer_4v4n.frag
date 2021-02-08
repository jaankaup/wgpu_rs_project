#version 450

layout(location = 0) in vec4 pos_f;
layout(location = 1) in vec4 nor_f;

layout(location = 0) out vec4 final_color;

layout(set=0, binding=0) uniform camerauniform {
    mat4 u_view_proj;
    vec4 camera_pos;
};

layout(set = 1, binding = 0) uniform texture2D t_diffuse1;
layout(set = 1, binding = 1) uniform sampler s_diffuse1;
layout(set = 1, binding = 2) uniform texture2D t_diffuse2;
layout(set = 1, binding = 3) uniform sampler s_diffuse2;

const vec3 light_pos = vec3(3.0, 3.0, 3.0);
const vec3 light_color = vec3(1.0, 1.0, 1.0);
const vec3 material_spec_color = vec3(1.0, 1.0, 1.0);
const float material_shininess = 170.0;
const float ambient_coeffience = 0.15;
const float attentuation_factor = 0.009;

void main() {

    vec3 light_dir = normalize(light_pos - pos_f.xyz);
    vec3 normal = normalize(nor_f).xyz; // is this necessery? 
    float diff_coeffient = max(0.0, dot(normal, light_dir));
    vec3 reflection_vector = reflect(-light_dir, normal);
    vec3 camera_dir = normalize(camera_pos.xyz - pos_f.xyz);

    float cosAngle = max(0.0, dot(camera_dir, reflection_vector));

    float specular_coeffient = 0.0;
    if (diff_coeffient > 0.0)
        specular_coeffient = pow(cosAngle, material_shininess);

    const float offset_factor = 1.5;

    const vec2 coord1 = pos_f.xy*offset_factor;
    const vec2 coord2 = pos_f.xz*offset_factor;
    const vec2 coord3 = pos_f.zz*offset_factor;

    const vec3 surfaceColor_grass = texture(sampler2D(t_diffuse1, s_diffuse1), offset_factor * (coord1 + coord2 + coord3) / 3.0).xyz;
    const vec3 surfaceColor_rock = texture(sampler2D(t_diffuse2, s_diffuse2), 1.1 * (coord1 + coord2 + coord3) / 3.0).xyz;
    const vec3 surface_color = mix(
        surfaceColor_rock, surfaceColor_grass,
        clamp(0.4*nor_f.x + 0.6*nor_f.y, 0.0, 1.0));

    //const vec3 surface_color = vec3(0.5, 0.5, 0.75);

    //vec3 surface_color = mix(surfaceColor_rock, surfaceColor_grass, clamp(0.4 * nor_f.x + 0.6 * nor_f.y, 0.0, 1.0)); 

//    const vec3 surface_color = texture(sampler2D(t_diffuse, s_diffuse), (coord1 + coord2 + coord3) / 3.0).xyz;

    vec3 specular_component = specular_coeffient * material_spec_color * light_color;
    vec3 ambient_component = ambient_coeffience * light_color * surface_color;
    vec3 diffuse_component = diff_coeffient * light_color * surface_color;

    float distance_to_light = distance(pos_f.xyz, light_pos); 
    float attentuation = 1.0 / (1.0 + attentuation_factor * pow(distance_to_light,2));

    final_color = vec4(ambient_component + attentuation * (diffuse_component + specular_component) , 1.0);
}
