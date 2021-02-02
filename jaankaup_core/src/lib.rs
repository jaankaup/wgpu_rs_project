pub mod wgpu_system; 
pub mod input; 
pub mod shader; 
pub mod misc; 
pub mod buffer; 
pub mod texture; 
pub mod assets; 
pub mod camera; 
pub mod two_triangles; 
pub mod mc; 
pub mod vvvvnnnn; 
pub mod temp; 
pub mod render_pipelines; 

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
