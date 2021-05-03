[[block]]
struct Dimensions {
    dim: [[stride(16)]] array<vec3<u32>>;
};

[[block]]
struct FutureUsage {
    something: [[stride(16)]] array<vec4<f32>>;
};

[[block]]
struct Output {
    output: [[stride(4)]] array<f32>;
};

// TODO: uniform. Compine binding 0 & 1
[[group(0), binding(0)]]
var<storage> the_number_of_workgroups: [[access(read)]] Dimensions;

// TODO: uniform.
[[group(0), binding(1)]]
var<storage> dimensions: [[access(read)]] Dimensions;

[[group(0), binding(2)]]
var<storage> future_usage1: [[access(read)]] FutureUsage;

[[group(0), binding(3)]]
var<storage> noise_output: [[access(write)]] Output;

// Noise functions copied from https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83 and converted to wgsl.

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 10000.0);
}

fn hash_v2(p: vec2<f32>) -> f32 {
    return fract(10000.0 * sin(17.0 * p.x + p.y * 0.1) * (0.1 + abs(sin(p.y * 13.0 + p.x))));
}

fn noise(x: f32) -> f32 {
    let i: f32 = floor(x);
    let f: f32 = fract(x);
    let u: f32 = f * f * (3.0 - 2.0 * f);
    return mix(hash(i), hash(i + 1.0), u);
}

fn noise2(x: vec2<f32>) -> f32 {

	let i: vec2<f32> = floor(x);
	let f: vec2<f32> = fract(x);

	// Four corners in 2D of a tile
	let a: f32 = hash_v2(i);
	let b: f32 = hash_v2(i + vec2<f32>(1.0, 0.0));
	let c: f32 = hash_v2(i + vec2<f32>(0.0, 1.0));
	let d: f32 = hash_v2(i + vec2<f32>(1.0, 1.0));

	let u: vec2<f32> = f * f * (3.0 - 2.0 * f);
	return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn noise3(x: vec3<f32>) -> f32 {

	let st = vec3<f32>(110.0, 241.0, 171.0);

	let i = floor(x);
	let f = fract(x);

    	let n = dot(i, st);


	let u = f * f * (3.0 - 2.0 * f);
	return mix(mix(mix( hash(n + dot(st, vec3<f32>(0.0, 0.0, 0.0))), hash(n + dot(st, vec3<f32>(1.0, 0.0, 0.0))), u.x),
                   mix( hash(n + dot(st, vec3<f32>(0.0, 1.0, 0.0))), hash(n + dot(st, vec3<f32>(1.0, 1.0, 0.0))), u.x), u.y),
               mix(mix( hash(n + dot(st, vec3<f32>(0.0, 0.0, 1.0))), hash(n + dot(st, vec3<f32>(1.0, 0.0, 1.0))), u.x),
                   mix( hash(n + dot(st, vec3<f32>(0.0, 1.0, 1.0))), hash(n + dot(st, vec3<f32>(1.0, 1.0, 1.0))), u.x), u.y), u.z);
}

let NUM_OCTAVES: u32 = 5u;

fn fbm(x: f32) -> f32 {

    var v: f32 = 0.0;
    var a: f32 = 0.5;
    var xx: f32 = x; 
    let shift: f32 = 100.0;
    for (var i: u32 = 0u; i < NUM_OCTAVES; i = i + 1u) {
    	v = a + a * noise(xx);
    	xx = xx * 2.0 + shift;
    	a = a * 0.5;
    }
    return v;
}


fn fbm2(x: vec2<f32>) -> f32 {

    let shift = vec2<f32>(100.0);
    let rot = mat2x2<f32>(vec2<f32>(cos(0.5), sin(0.5)), vec2<f32>(-sin(0.5), cos(0.50)));
    
    var v: f32 = 0.0;
    var a: f32 = 0.5;
    var xx: vec2<f32> = x; 
    
    for (var i: u32 = 0u; i < NUM_OCTAVES; i = i + 1u) {
        v = v + a * noise2(xx);
        xx = rot * xx * 2.0 + shift;
        a = a * 0.5;
    }
    return v;
}

fn fbm3(x: vec3<f32>) -> f32 {

    let shift: f32 = 100.0;

    var v: f32 = 0.0;
    var a: f32 = 0.5;
    var xx: vec3<f32> = x; 

    for (var i: u32 = 0u; i < NUM_OCTAVES; i = i + 1u) {
    	v = a + a * noise3(xx);
    	xx = xx * 2.0 + shift;
    	a = a * 0.5;
    }
    return v;
}

fn index1D_to_index3D(global_index: vec3<u32>) -> vec3<u32> {
	var index: u32 = global_index.x;
	var wh: u32 = dimensions.dim[0].x * dimensions.dim[0].y;
	let z: u32 = index / wh;
	index = index - z * wh;
	let y: u32 = index / dimensions.dim[0].x;
	index = index - y * dimensions.dim[0].x;
	let x: u32 = index;
	return vec3<u32>(x, y, z);	
}

[[stage(compute), workgroup_size(64,1,1)]]
fn main([[builtin(local_invocation_id)]] local_id: vec3<u32>,
        [[builtin(local_invocation_index)]] local_index: u32,
        [[builtin(global_invocation_id)]] global_id: vec3<u32>) {

  let coordinate3D = index1D_to_index3D(global_id);
 
  let fx = f32(coordinate3D.x) * 0.1;
  let fy = f32(coordinate3D.y) * 0.1;
  let fz = f32(coordinate3D.z) * 0.1;
 
  let noise_a = noise3(vec3<f32>(1.5 + future_usage1.something[0].x) + vec3<f32>(fz, fy, fx));
  let result_value = fy - 1.9 - 0.2 * cos(3.0 * future_usage1.something[0].x + fy) + 0.1 * sin(1.5 * fz + noise_a * 0.5) + 0.3 * noise_a;

  noise_output.output[global_id.x] = result_value;
}
