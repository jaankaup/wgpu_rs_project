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
use jaankaup_core::noise3d::*;

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
}

// State for this application.
struct HelloApp {
    textures: HashMap<String, JTexture>, 
    buffers: HashMap<String, wgpu::Buffer>,
    two_triangles: TwoTriangles,
    two_triangles_bind_group: wgpu::BindGroup,
    depth_texture: JTexture,
    camera: Camera,
    mc: MarchingCubes,
    mc_slime: MarchingCubes,
    test_layout: TestLayoutEntry,
    bind: Vec<wgpu::BindGroup>,
    bind_slime: Vec<wgpu::BindGroup>,
    draw_count_mc: u32,
    draw_count_mc_slime: u32,
    mc_params_slime: McParams,
    //shaders: HashMap<String, ShaderModule>,
    //render_passes: HashMap<String, RenderPass>,
}

impl HelloApp {

    fn create_textures(configuration: &WGPUConfiguration) -> (JTexture, JTexture, JTexture, JTexture) {
        let grass_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../textures/grass2.png")[..],
            None);
        let rock_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../textures/rock.png")[..],
            None);
        let slime_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../textures/slime.png")[..],
            None);
        let slime_texture2 = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../textures/slime2.png")[..],
            None);
        (grass_texture, rock_texture, slime_texture, slime_texture2)
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
        let (grass_texture, rock_texture, slime, slime2) = HelloApp::create_textures(&configuration); 
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
        textures.insert("rock".to_string(), rock_texture); 
        textures.insert("slime".to_string(), slime); 
        textures.insert("slime2".to_string(), slime2); 

        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);
        log::info!("Aspect ratio == {} / {}", configuration.size.width as f32, configuration.size.height as f32);

        //let _ = camera.get_camera_uniform(&configuration.device);
        // let _ = camera.get_ray_camera_uniform(&configuration.device);

        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/renderer_4v4n.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/renderer_4v4n.frag.spv");

        // Render pipeline...
        let t = TestLayoutEntry::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src)
        );
        // Check the correspondence of resources and the pipeline interface.
        check_correspondence(
            &t.layout_entries,
            &vec![
                vec![wgpu::BindingResource::Buffer {
                    buffer: &camera.get_camera_uniform(&configuration.device),
                    offset: 0,
                    size: None,
                }], 
                vec![wgpu::BindingResource::TextureView(&textures.get("grass").unwrap().view),
                     wgpu::BindingResource::Sampler(&textures.get("grass").unwrap().sampler),
                     wgpu::BindingResource::TextureView(&textures.get("rock").unwrap().view),
                     wgpu::BindingResource::Sampler(&textures.get("rock").unwrap().sampler)
                ]
            ]
        );
        // Create bind groups for basic render pipeline and grass/rock textures. 
        let t_bindgroups = create_bind_groups(
                                &configuration.device, 
                                &t.layout_entries,
                                &vec![
                                    vec![&wgpu::BindingResource::Buffer {
                                            buffer: &camera.get_camera_uniform(&configuration.device),
                                            offset: 0,
                                            size: None,
                                    }], 
                                    vec![&wgpu::BindingResource::TextureView(&textures.get("grass").unwrap().view),
                                         &wgpu::BindingResource::Sampler(&textures.get("grass").unwrap().sampler),
                                         &wgpu::BindingResource::TextureView(&textures.get("rock").unwrap().view),
                                         &wgpu::BindingResource::Sampler(&textures.get("rock").unwrap().sampler)]
                                ]
        );

        // Create bind groups for basic render pipeline and slime/slime2 textures. 
        let t_slime_bindgroups = create_bind_groups(
                                     &configuration.device, 
                                     &t.layout_entries,
                                     &vec![
                                         vec![&wgpu::BindingResource::Buffer {
                                                 buffer: &camera.get_camera_uniform(&configuration.device),
                                                 offset: 0,
                                                 size: None,
                                         }], 
                                         vec![&wgpu::BindingResource::TextureView(&textures.get("slime").unwrap().view),
                                              &wgpu::BindingResource::Sampler(&textures.get("slime").unwrap().sampler),
                                              &wgpu::BindingResource::TextureView(&textures.get("slime2").unwrap().view),
                                              &wgpu::BindingResource::Sampler(&textures.get("slime2").unwrap().sampler)]
                                     ]
        );

        // The environment (mountains marching cubes).
        let mc = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test.comp.spv"))
        );

        // Create output buffer for "mountains", the output of mc.
        buffers.insert(
            "mc_output".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &vec![0 as f32 ; 128*128*128*24],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        // Create parameters for "mountain" marching cubes.
        let mut mc_params = McParams::init(
                &configuration.device, 
                &cgmath::Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0),
                0.0,
                0.05
        );

        // Add bindings to the mc.
        let mc_bind_groups = mc.create_bind_groups(
            &configuration.device,
            &mc_params,
            &buffers.get("mc_output").unwrap()
        );

        // Add create bind groups to the mc_params.
        mc_params.bind_groups = Some(mc_bind_groups); 
        
        // The slime marching cubes.
        let mc_slime = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test_slime.comp.spv"))
        );

        // Create output buffer for slime triangle mesh (the output of slime mc).
        buffers.insert(
            "mc_output_slime".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            &vec![0 as f32 ; 64*64*64*24],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        // Create parameters for "slime" marching cubes.
        let mut mc_params_slime = McParams::init(
                &configuration.device, 
                &cgmath::Vector4::<f32>::new(0.0, 1.0, 0.0, 1.0),
                0.0,
                0.05
        );

        // Create bind groups for slime.
        let mc_bind_groups_slime = mc_slime.create_bind_groups(
            &configuration.device,
            &mc_params_slime,
            &buffers.get("mc_output_slime").unwrap()
        );

        // Add create bind groups to the mc_slime.
        mc_params_slime.bind_groups = Some(mc_bind_groups_slime); 

        // Create nouse 3d "texture".
        
        let shader_comp_3d_tex = wgpu::include_spirv!("../../shaders/spirv/data3d_test.comp.spv");
        log::info!("Creating 3d.");
        let texture3D = Custom3DTexture::init(
                &configuration.device,
                &configuration.device.create_shader_module(&shader_comp_3d_tex)
        );
        log::info!("Finished creating 3d.");

        // Perform both mountain and slime marching cubes.
        let mut encoder = configuration.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Juuu") });

        mc.dispatch(&mc_params.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    128,
                    64
        ); 

        mc_slime.dispatch(&mc_params_slime.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    6,
                    64
        ); 

        configuration.queue.submit(Some(encoder.finish()));

        // TODO. Figure out how to do this with wasm.
        let mut k: Vec<u32>;
        let mut k_slime: Vec<u32>;

        //#[cfg(target_arch = "wasm32")] {
        //k =  to_vec::<u32>(&configuration.device,
        //            &configuration.queue,
        //            &mc_params.counter_buffer,
        //            0 as wgpu::BufferAddress,
        //            4 as wgpu::BufferAddress).await;
        //log::info!("Mc counter == {}", k[0]);
        //}
        // wasm_bindgen_futures::spawn_local(async move {
        //     k =  to_vec::<u32>(&configuration.device,
        //                 &configuration.queue,
        //                 &mc_params.counter_buffer,
        //                 0 as wgpu::BufferAddress,
        //                 4 as wgpu::BufferAddress).await;
        //     log::info!("Mc counter == {}", k[0]);
        // });

        #[cfg(not(target_arch = "wasm32"))]
        let k =  pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &mc_params.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));
        #[cfg(not(target_arch = "wasm32"))]
        log::info!("Mc counter == {}", k[0]);

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            k_slime =  to_vec::<u32>(&configuration.device,
                                &configuration.queue,
                                &mc_params_slime.counter_buffer,
                                0 as wgpu::BufferAddress,
                                4 as wgpu::BufferAddress).await;
            log::info!("Mc counter_slime == {}", k_slime[0]);
        });

        #[cfg(not(target_arch = "wasm32"))]
        let k_slime =  pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &mc_params_slime.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));

        #[cfg(not(target_arch = "wasm32"))]
        log::info!("Mc counter_slime == {}", k_slime[0]);

        // let k2 =  pollster::block_on(to_vec::<f32>(&configuration.device,
        //                                       &configuration.queue,
        //                                       &buffers.get("mc_output").unwrap(),
        //                                       0 as wgpu::BufferAddress,
        //                                       (k[0] * 8 * std::mem::size_of::<f32>() as u32) as wgpu::BufferAddress));
        // log::info!("k2.len() == {}", k2.len());
        // for i in 0..k[0] * 8 {
        //     if i % 4 == 0 { println!(""); } 
        //     print!("{} ", k2[i as usize]);
        // }

        HelloApp {
            textures: textures,
            buffers: buffers,
            two_triangles: two_triangles,
            two_triangles_bind_group: bind_group,
            depth_texture: depth_texture,
            camera: camera,
            mc: mc,
            mc_slime: mc_slime,
            test_layout: t,
            bind: t_bindgroups,
            bind_slime: t_slime_bindgroups,
            draw_count_mc: k[0],
            draw_count_mc_slime: k_slime[0],
            mc_params_slime: mc_params_slime,
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
             &self.buffers.get("mc_output").unwrap(),
             0..self.draw_count_mc, 
             true
        );

        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.bind_slime,
             &self.test_layout.pipeline,
             &self.buffers.get("mc_output_slime").unwrap(),
             0..self.draw_count_mc_slime, 
             false
        );

        // self.two_triangles.draw(&mut encoder, &frame, &self.depth_texture, &self.two_triangles_bind_group, true);

        queue.submit(Some(encoder.finish()));
    }

    fn input(&mut self, queue: &wgpu::Queue, input_cache: &InputCache) {
        self.camera.update_from_input(&queue, &input_cache);
    }

    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, _new_size: winit::dpi::PhysicalSize<u32>) {
        self.depth_texture = JTexture::create_depth_texture(&device, &sc_desc, Some("depth-texture"));
        self.camera.resize(sc_desc.width as f32, sc_desc.height as f32);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, input: &InputCache) {
        
        self.mc_params_slime.reset_counter(&queue);
        self.mc_params_slime.update_params(
            &queue,
            &None,
            &Some(0.15 * (((input.get_time() / 30000000) as f32) * 0.005).sin()),
            &None,
            &Some(((input.get_time() / 30000000) as f32) * 0.0015),
        ); 

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Juuu") });

        self.mc_slime.dispatch(&self.mc_params_slime.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    6,
                    64
        );

        queue.submit(Some(encoder.finish()));

        #[cfg(not(target_arch = "wasm32"))]
        let k_slime =  pollster::block_on(to_vec::<u32>(&device,
                                              &queue,
                                              &self.mc_params_slime.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));

        self.draw_count_mc_slime = k_slime[0];
    }
}

fn main() {
    
    ws::run_loop::<HelloApp, BasicLoop, MyFeatures>(); 
    println!("Finished...");
}
