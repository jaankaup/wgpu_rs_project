use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};
use jaankaup_core::buffer::*;
use jaankaup_core::texture::Texture as JTexture;
use jaankaup_core::two_triangles::*;
use jaankaup_core::mc::*;
use jaankaup_core::camera::{Camera};
use jaankaup_core::input::InputCache;
use jaankaup_core::render_pipelines::*;

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
}

// State for this application.
struct HelloApp {
    _textures: HashMap<String, JTexture>, 
    buffers: HashMap<String, wgpu::Buffer>,
    two_triangles: TwoTriangles,
    two_triangles_bind_group: wgpu::BindGroup,
    depth_texture: JTexture,
    camera: Camera,
    mc: MarchingCubes,
    test_layout: TestLayoutEntry,
    bind: Vec<wgpu::BindGroup>,
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
        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
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
        buffers.insert(
            "two".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &[-1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
               1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
               1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
               1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
              -1.0,  1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
              -1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
            ],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC,
            None
            )
        );

        textures.insert("grass".to_string(), grass_texture); 

        let camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);

        //let _ = camera.get_camera_uniform(&configuration.device);
        // let _ = camera.get_ray_camera_uniform(&configuration.device);

        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.frag.spv");

        let t = TestLayoutEntry::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src)
        );
        check_correspondence(
            &t.layout_entries,
            &vec![vec![wgpu::BindingResource::TextureView(&textures.get("grass").unwrap().view),
                                      wgpu::BindingResource::Sampler(&textures.get("grass").unwrap().sampler)]]
        );
        let t_bindgroups = create_bind_groups(
                                &configuration.device, 
                                &t.layout_entries,
                                &vec![vec![&wgpu::BindingResource::TextureView(&textures.get("grass").unwrap().view),
                                      &wgpu::BindingResource::Sampler(&textures.get("grass").unwrap().sampler)]]
        );

        //create_bind_groups(
        //    &t.entry_layout,
        //    &vec![
        //        vec![&wgpu::BindingResource::TextureView(&textures.get("grass").unwrap().view),
        //             &wgpu::BindingResource::Sampler(&textures.get("grass").unwrap().sampler)]
        //    ]);

        let mc = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test.comp.spv"))
        );

        buffers.insert(
            "mc_output".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &vec![0 as f32 ; 64*64*64*24],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        let mut mc_params = McParams::init(
                &configuration.device, 
                &cgmath::Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0),
                0.0,
                0.5
        );

        let mc_bind_groups = mc.create_bind_groups(
            &configuration.device,
            &mc_params,
            &buffers.get("mc_output").unwrap()
        );

        mc_params.bind_groups = Some(mc_bind_groups); 
        
        let mut encoder = configuration.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Juuu") });

        mc.dispatch(&mc_params.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    32,
                    32,
                    32
        ); 

        configuration.queue.submit(Some(encoder.finish()));

        let k =  pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &mc_params.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));
        println!("yeaaaaah");
        log::info!("Mc counter == {}", k[0]);
        // let k =  to_vec::<u32>(&configuration.device,
        //                        &configuration.queue,
        //                        &mc_params.counter_buffer,
        //                        0 as wgpu::BufferAddress,
        //                        4 as wgpu::BufferAddress);
        // println!("yeaaaaah");
        // log::info!("Mc counter == {}", k.poll());
// pub async fn to_vec<T: Convert2Vec>(
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     buffer: &wgpu::Buffer,
//     _src_offset: wgpu::BufferAddress,
//     copy_size: wgpu::BufferAddress,
//     ) -> Vec<T> {

        HelloApp {
            _textures: textures,
            buffers: buffers,
            two_triangles: two_triangles,
            two_triangles_bind_group: bind_group,
            depth_texture: depth_texture,
            camera: camera,
            mc: mc,
            test_layout: t,
            bind: t_bindgroups,
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
                log::info!("FAILED");
                *swap_chain = device.create_swap_chain(surface, sc_desc);
                swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture").output
            },
        };

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
        });

        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.bind,
             &self.test_layout.pipeline,
             &self.buffers.get("two").unwrap(),
             (0..6), 
             true
        );

        //self.two_triangles.draw(&mut encoder, &frame, &self.depth_texture, &self.two_triangles_bind_group, true);

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
