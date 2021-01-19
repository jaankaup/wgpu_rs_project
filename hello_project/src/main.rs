use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};
use jaankaup_core::shader::ShaderModule;
use jaankaup_core::pipeline::{BindGroupInfo, RenderPipelineInfo, Resource};
use jaankaup_core::buffer::*;
use jaankaup_core::texture::Texture as JTexture;
use jaankaup_core::two_triangles::*;
//use jaankaup_core::assets::create_screen_texture_buffer;

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
}

// State for this application.
struct HelloApp {
    textures: HashMap<String, JTexture>, 
    buffers: HashMap<String, wgpu::Buffer>,
    //shaders: HashMap<String, ShaderModule>,
    //render_passes: HashMap<String, RenderPass>,
}

impl Application for HelloApp {

    fn init(configuration: &WGPUConfiguration) -> Self {
        
        // Create buffer container.
        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();

        //let screen_buffer = create_screen_texture_buffer(&configuration.device);
        //buffers.insert("screen".to_string(),screen_buffer);

        let two_triangles = TwoTriangles::init(&configuration.device, &configuration.sc_desc);

        HelloApp { 
            textures: HashMap::<String, JTexture>::new(),
            buffers: buffers,
            //shaders: load_shaders(&configuration.device),
            //render_passes: HashMap::<String, RenderPass>::new(),
        }
    }

    fn render(self) {

    }

    fn input(self) {

    }

    fn resize(self) {

    }

    fn update(self) {

    }

}

//fn create_screen_texture_render_pass(

// fn load_shaders(device: &wgpu::Device) -> HashMap<String,ShaderModule> {
// 
//     let mut hmap = HashMap::<String, ShaderModule>::new();
//     
//     println!("Creating vertex module 'screen texture shader' ...");
//     let screen_module_vert = ShaderModule::build(
//         &"screen_vert".to_string(),
//         &wgpu::include_spirv!("../../shaders/spirv/screen_texture.vert.spv"),
//         &device
//     );
// 
//     println!("Creating fragment module 'screen texture shader' ...");
//     let screen_module_vert = ShaderModule::build(
//         &"screen_frag".to_string(),
//         &wgpu::include_spirv!("../../shaders/spirv/screen_texture.frag.spv"),
//         &device
//     );
//     hmap
// }

// Create a pipeline and binding groups for shader that renders a given texture to the screen.
// Should be used with two_triangles buffer.
//fn create_screen_texture_info(texture_name: &'static str, sample_count: u32) -> RenderPipelineInfo {
//    use jaankaup_core::misc as jm;
//
//    let screen_texture_info: RenderPipelineInfo = RenderPipelineInfo {
//        vertex_shader: "screen_vert".to_string(),
//        fragment_shader: Some("sreen_frag"),
//        bind_groups: vec![
//                vec![
//                    BindGroupInfo {
//                        binding: 0,
//                        visibility: wgpu::ShaderStage::FRAGMENT,
//                        resource: Resource::TextureView(texture_name),
//                        binding_type: wgpu::BindingType::Texture {
//                           sample_type: wgpu::TextureSampleType::Float {filterable: true},
//                           view_dimension: wgpu::TextureViewDimension::D2,
//                           multisampled: jm::multisampled(sample_count),
//                        },
//                    },
//                    BindGroupInfo {
//                        binding: 1,
//                        visibility: wgpu::ShaderStage::FRAGMENT,
//                        resource: Resource::TextureSampler(texture_name),
//                        binding_type: wgpu::BindingType::Sampler {
//                           comparison: false,
//                           filtering: true,
//                        },
//                    },
//                ],
//        ],
//        input_formats: vec![
//            (wgpu::VertexFormat::Float4, 4 * std::mem::size_of::<f32>() as u64),
//            (wgpu::VertexFormat::Float4, 4 * std::mem::size_of::<f32>() as u64)
//        ],
//    };
//    screen_texture_info
//}


fn main() {
    
    ws::run_loop::<HelloApp, BasicLoop, MyFeatures>(); 
    println!("Finished...");
}
