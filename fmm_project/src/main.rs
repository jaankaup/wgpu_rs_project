use jaankaup_core::buffer::buffer_from_data;
use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};
use jaankaup_core::texture::Texture as JTexture;
use jaankaup_core::camera::{Camera};
use jaankaup_core::input::InputCache;
use model_loader::load_triangles_from_obj;

// Redefine needed features for this application.
struct FMM_Features {}
impl WGPUFeatures for FMM_Features {
}

// The fmm application.
struct FMM_App {
    depth_texture: JTexture,
    camera: Camera,
    buffers: HashMap<String, wgpu::Buffer>,
}

impl FMM_App {

}

impl Application for FMM_App {

    /// Initialize fmm application.
    fn init(configuration: &WGPUConfiguration) -> Self {

        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();

        // Create the depth texture for fmm application.
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("fmm depth texture")
        ); 

        // Initialize camera for fmm application.
        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);

        Self {
            depth_texture,
            camera,
            buffers,
        }
    }
    fn render(&mut self,
              device: &wgpu::Device,
              queue: &mut wgpu::Queue,
              swap_chain: &mut wgpu::SwapChain,
              surface: &wgpu::Surface,
              sc_desc: &wgpu::SwapChainDescriptor) {
    }

    fn input(&mut self, queue: &wgpu::Queue, input_cache: &InputCache) {
        self.camera.update_from_input(&queue, &input_cache);
    }

    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, _new_size: winit::dpi::PhysicalSize<u32>) {
        self.depth_texture = JTexture::create_depth_texture(&device, &sc_desc, Some("depth-texture"));
        self.camera.resize(sc_desc.width as f32, sc_desc.height as f32);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, input: &InputCache) {

    }
}

fn create_fmm_buffer(device: &wgpu::Device,
                     name: String, 
                     dimension: [u32; 3],
                     element_size: u32,
                     buffers: &mut HashMap<String, wgpu::Buffer>) {

        assert!(dimension[0] % 4 && dimension[1] % 4 && dimension[2] % 4, "Each dimension should be a multiple of 4.");  

        buffers.insert(
            name,
            buffer_from_data::<f32>(
            &device,
            &vec![0 as f32 ; dimension[0] as usize * dimension[1] as usize * dimension[2] as usize],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );
}

fn main() {
    ws::run_loop::<FMM_App, BasicLoop, FMM_Features>(); 
}
