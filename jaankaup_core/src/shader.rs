/// A struct for shader module.
pub struct ShaderModule {
    pub id: String,
    pub module: wgpu::ShaderModule,
}

impl ShaderModule {
    /// Compile shader module from given source.
    pub fn build(id: &String, spirv_src: wgpu::ShaderModuleSource, device: &wgpu::Device) -> Self {

        let temp = device.create_shader_module(spirv_src); 

        Self {
            id: id.to_string(),
            module: temp, 
        }
    }
}
