use std::collections::HashMap;
//use winit::dpi::PhysicalSize;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};
use jaankaup_core::texture::Texture as JTexture;
use jaankaup_core::two_triangles::*;
use jaankaup_core::mc::*;
use jaankaup_core::camera::{Camera};
use jaankaup_core::input::InputCache;
//use jaankaup_core::assets::create_screen_texture_buffer;

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
}

// State for this application.
struct HelloApp {
    _textures: HashMap<String, JTexture>, 
    _buffers: HashMap<String, wgpu::Buffer>,
    two_triangles: TwoTriangles,
    two_triangles_bind_group: wgpu::BindGroup,
    depth_texture: JTexture,
    camera: Camera,
    mc: MarchingCubes,
    //shaders: HashMap<String, ShaderModule>,
    //render_passes: HashMap<String, RenderPass>,
}

impl HelloApp {

    fn create_texture(configuration: &WGPUConfiguration) -> JTexture {
        let grass_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../textures/grass2.png")[..],
            None);
        grass_texture
    }
}

impl Application for HelloApp {

    fn init(configuration: &WGPUConfiguration) -> Self {
        
        // Create buffer container.
        let buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
        let mut textures: HashMap<String, JTexture> = HashMap::new();

        //let screen_buffer = create_screen_texture_buffer(&configuration.device);
        //buffers.insert("screen".to_string(),screen_buffer);

        let two_triangles = TwoTriangles::init(&configuration.device, &configuration.sc_desc);
        let grass_texture = HelloApp::create_texture(&configuration); 
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("depth_texture")
        ); 
        let bind_group = TwoTriangles::create_bind_group(
            &configuration.device,
            &grass_texture
        );

        textures.insert("grass".to_string(), grass_texture); 

        let camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);

        //let _ = camera.get_camera_uniform(&configuration.device);
        // let _ = camera.get_ray_camera_uniform(&configuration.device);
        let mc = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test.comp.spv"))
        );

        HelloApp { 
            _textures: textures,
            _buffers: buffers,
            two_triangles: two_triangles,
            two_triangles_bind_group: bind_group,
            depth_texture: depth_texture,
            camera: camera,
            mc: mc,
            //shaders: load_shaders(&configuration.device),
            //render_passes: HashMap::<String, RenderPass>::new(),
        }
    }

    fn render(&mut self,
              device: &wgpu::Device,
              queue: &mut wgpu::Queue,
              swap_chain: &mut wgpu::SwapChain,
              surface: &wgpu::Surface,
              sc_desc: &wgpu::SwapChainDescriptor) {

        let frame = match swap_chain.get_current_frame() {
            Ok(frame) => { frame.output },
            Err(_) => {
                println!("FAILED");
                *swap_chain = device.create_swap_chain(surface, sc_desc);
                swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture").output
            },
        };

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
        });

        self.two_triangles.draw(&mut encoder, &frame, &self.depth_texture, &self.two_triangles_bind_group, true); 

        queue.submit(Some(encoder.finish()));
    }

    fn input(&mut self, input_cache: &InputCache) {
        self.camera.update_from_input(&input_cache);
    }

    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, _new_size: winit::dpi::PhysicalSize<u32>) {
        self.depth_texture = JTexture::create_depth_texture(&device, &sc_desc, Some("depth-texture"));
    }

    fn update(&self) {

    }
}

fn main() {
    
    ws::run_loop::<HelloApp, BasicLoop, MyFeatures>(); 
    println!("Finished...");
}
