use bytemuck::{Pod, Zeroable};
use cgmath::{prelude::*, Vector3};

/// A trait for things that can copy and convert a wgpu-rs buffer to
/// a std::Vec. 
pub trait Convert2Vec where Self: std::marker::Sized {
    fn convert(data: &[u8]) -> Vec<Self>;  
}

/// A macro for creating Convert2Vec for specific a primitive 
/// number type. Note that the type must implement from_ne_bytes.
/// This works only in async functions. This cannot be used
/// in winit event_loop! Use it before entering event_loop.
macro_rules! impl_convert {
  ($to_type:ty) => {
    impl Convert2Vec for $to_type {
      fn convert(data: &[u8]) -> Vec<Self> {
            let result = data
                .chunks_exact(std::mem::size_of::<Self>())
                .map(|b| *bytemuck::try_from_bytes::<Self>(b).unwrap())
                .collect();
            result
      }
    }
  }
}

// TODO: keep only those that are needed.
impl_convert!{Vertex_vvvvnnnn}
impl_convert!{f32}
impl_convert!{u32}
impl_convert!{u8}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex_vvvc {
    pub position: [f32 ; 3],
    pub color: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex_vvvcnnnn {
    pub position: [f32 ; 3],
    pub color: u32,
    pub normal: [f32 ; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex_vvvvnnnn {
    pub position: [f32 ; 4],
    pub normal: [f32 ; 4],
}

/// Calculate normal from three vertices (triangle vertices).
pub fn calculate_normal(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>) -> Vector3<f32> {
    let u = b - a;
    let v = c - a;
    let result = u.cross(v).normalize();
    result
}

/// Clamp function.
pub fn clamp(val: f32, min: f32, max: f32) -> f32 {
    let result  = if val >= max { max } else { val };
    let result2 = if result <= min { min } else { val };
    result2
}

/// Encode vector to "rgba" uint. 
/// Conversion: vec4(x,y,z,w) => Uint(xxyyzzww); 
pub fn encode_rgba_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
  ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32

}

/// [a1..a2] -> [b1..b2]. s value to scale.
pub fn map_range(a1: f32, a2: f32, b1: f32, b2: f32, s: f32) -> f32 {
    b1 + (s - a1) * (b2 - b1) / (a2 - a1)
}

pub fn multisampled(sample_count: u32) -> bool {
  match sample_count { 1 => false, 2 => true, 4 => true, 8 => true, 16 => true, _ => panic!("Illegal sample count {}.", sample_count) }
}

/// Takes wgpu::VertexFormats as input and returns (stride, Vec<wgpu::VertexBufferDescriptor>)
pub fn create_vb_descriptor(formats: &Vec<wgpu::VertexFormat>) -> (u64, Vec<wgpu::VertexAttributeDescriptor>) { 

    let mut attribute_descriptors: Vec<wgpu::VertexAttributeDescriptor> = Vec::new();
    let mut stride: u64 = 0;
    for (i, format) in formats.iter().enumerate() {
        let size = match format {
                wgpu::VertexFormat::Uchar2 => 2 * std::mem::size_of::<u8>() as u64, 
                wgpu::VertexFormat::Uchar4 => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Char2 => 2 * std::mem::size_of::<i8>() as u64,
                wgpu::VertexFormat::Char4 => 4 * std::mem::size_of::<i8>() as u64,
                wgpu::VertexFormat::Uchar2Norm => 2 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Uchar4Norm => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Char2Norm => 2 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Char4Norm => 4 * std::mem::size_of::<u8>() as u64,
                wgpu::VertexFormat::Ushort2 => 2 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Ushort4 => 4 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Short2 => 2 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Short4 => 4 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Ushort2Norm => 2 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Ushort4Norm => 4 * std::mem::size_of::<u16>() as u64,
                wgpu::VertexFormat::Short2Norm => 2 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Short4Norm => 4 * std::mem::size_of::<i16>() as u64,
                wgpu::VertexFormat::Half2 => unimplemented!(),
                wgpu::VertexFormat::Half4 => unimplemented!(),
                wgpu::VertexFormat::Float => std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float2 => 2 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float3 => 3 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Float4 => 4 * std::mem::size_of::<f32>() as u64,
                wgpu::VertexFormat::Uint => std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint2 => 2 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint3 => 3 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Uint4 => 4 * std::mem::size_of::<u32>() as u64,
                wgpu::VertexFormat::Int => std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Int2 => 2 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Int3 => 3 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Int4 => 4 * std::mem::size_of::<i32>() as u64,
                wgpu::VertexFormat::Double
                | wgpu::VertexFormat::Double2
                | wgpu::VertexFormat::Double3
                | wgpu::VertexFormat::Double4
                => panic!("VERTEX_ATTRIBUTE_64BIT must be enabled to use Double formats")
        };
        attribute_descriptors.push(
            wgpu::VertexAttributeDescriptor {
                format: *format,
                offset: stride,
                shader_location: i as u32, 
            }
        );
        stride = stride + size;
    }

    (stride, attribute_descriptors)
}

///// Input: Vec<(u32, wgpu::BindingResource)>
//pub fn create_bindgroup(
//    device: &wgpu::Device,
//    binding_resources: &Vec<wgpu::BindGroupLayoutEntry>,
//    ) -> wgpu::BindGroup {
//
//    // Create wgpu::BindGroupLayoutEntry
//    //let mut bind_group_layouts: Vec<wgpu::BindGroupEntry> = Vec::new();
//
//    let bind_group_layout = 
//        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//            entries: Borrowed(&layout_entries),
//            label: None,
//        });
//
//
//
//
//    // Create wgpu::BindGroupLayout
//
//    // Create wgpu::BindGroupLayout
//
//    // Create wgpu::BindGroupDescriptor
//
//    // Create wgpu::BindGroup
//}

    

//pub enum VertexFormat {
//    Uchar2,
//    Uchar4,
//    Char2,
//    Char4,
//    Uchar2Norm,
//    Uchar4Norm,
//    Char2Norm,
//    Char4Norm,
//    Ushort2,
//    Ushort4,
//    Short2,
//    Short4,
//    Ushort2Norm,
//    Ushort4Norm,
//    Short2Norm,
//    Short4Norm,
//    Half2,
//    Half4,
//    Float,
//    Float2,
//    Float3,
//    Float4,
//    Uint,
//    Uint2,
//    Uint3,
//    Uint4,
//    Int,
//    Int2,
//    Int3,
//    Int4,
//}
