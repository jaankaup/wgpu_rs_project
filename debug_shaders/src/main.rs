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

// Redefine needed features for this application.
struct Debug_Features {}
impl WGPUFeatures for Debug_Features {
}

// The fmm application.
struct Debug_App {
    depth_texture: JTexture,
    camera: Camera,
}

impl Debug_App {

}

impl Application for Debug_App {

    /// Initialize fmm application.
    fn init(configuration: &WGPUConfiguration) -> Self {

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

fn main() {
    ws::run_loop::<Debug_App, BasicLoop, Debug_Features>(); 
}
