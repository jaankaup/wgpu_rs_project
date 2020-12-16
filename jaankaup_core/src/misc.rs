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