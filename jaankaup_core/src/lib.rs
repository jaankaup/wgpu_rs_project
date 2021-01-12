pub mod wgpu_system; 
pub mod input; 
pub mod shader; 
pub mod misc; 
pub mod buffer; 
pub mod texture; 
pub mod pipeline; 
pub mod assets; 
pub mod temp; 

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
