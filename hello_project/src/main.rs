//use byteorder::{BigEndian, ReadBytesExt};
use std::borrow::Cow;
use jaankaup_core::wgpu;
use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};
//use glsl_to_spirv;
use jaankaup_core::buffer::*;
use jaankaup_core::texture::Texture as JTexture;
//use jaankaup_core::two_triangles::*;
use jaankaup_core::mc::*;
use jaankaup_core::camera::{Camera};
use jaankaup_core::input::InputCache;
use jaankaup_core::render_pipelines::{
    draw,
    create_bind_groups,
    TestLayoutEntry,
    // check_correspondence,
};
use jaankaup_core::noise3d::*;

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::SPIRV_SHADER_PASSTHROUGH
        //wgpu::Features::ALL_NATIVE
    }
    fn required_limits() -> wgpu::Limits {
        let mut limits = wgpu::Limits::default();
        limits.max_storage_buffers_per_shader_stage = 8;
        limits
    }
}

// State for this application.
struct HelloApp {
    textures: HashMap<String, JTexture>, 
    buffers: HashMap<String, wgpu::Buffer>,
    //two_triangles: TwoTriangles,
    //two_triangles_bind_group: wgpu::BindGroup,
    depth_texture: JTexture,
    camera: Camera,
    mc_slime: MarchingCubes,
    test_layout: TestLayoutEntry,
    bind: Vec<wgpu::BindGroup>,
    bind_slime: Vec<wgpu::BindGroup>,
    draw_count_mc: u32,
    draw_count_mc_slime: u32,
    mc_params_slime: McParams,
    slime_texture3d_bindgroups: Vec<wgpu::BindGroup>,
    custom_3d: Custom3DTexture,
    //shaders: HashMap<String, ShaderModule>,
    //render_passes: HashMap<String, RenderPass>,
}

impl HelloApp {

    fn create_textures(configuration: &WGPUConfiguration) -> (JTexture, JTexture, JTexture, JTexture) {
        log::info!("Creating textures.");
        let grass_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../assets/textures/grass2.png")[..],
            None);
        let rock_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../assets/textures/rock.png")[..],
            None);
        let slime_texture = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            &include_bytes!("../../assets/textures/lava.png")[..],
            //&include_bytes!("../../assets/textures/slime.png")[..],
            None);
        let slime_texture2 = JTexture::create_from_bytes(
            &configuration.queue,
            &configuration.device,
            &configuration.sc_desc,
            1,
            //&include_bytes!("../../assets/textures/slime2.png")[..],
            //&include_bytes!("../../assets/textures/xXqQP0.png")[..],
            &include_bytes!("../../assets/textures/luava.png")[..],
            None);
        log::info!("Textures created OK.");
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

        //let two_triangles = TwoTriangles::init(&configuration.device, &configuration.sc_desc);
        let (grass_texture, rock_texture, slime, slime2) = HelloApp::create_textures(&configuration); 
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("depth_texture")
        ); 
        // let bind_group = TwoTriangles::create_bind_group(
        //     &configuration.device,
        //     &grass_texture
        // );
        // buffers.insert(
        //     "two".to_string(),
        //     buffer_from_data::<f32>(
        //     &configuration.device,
        //     // gl_Position     |    point_pos
        //     &[-1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        //        1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
        //        1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
        //        1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
        //       -1.0,  1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        //       -1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        //     ],
        //     wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
        //     None
        //     )
        // );

        textures.insert("grass".to_string(), grass_texture); 
        textures.insert("rock".to_string(), rock_texture); 
        textures.insert("slime".to_string(), slime); 
        textures.insert("slime2".to_string(), slime2); 

        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);
        camera.set_rotation_sensitivity(0.2);
        //camera.set_movement_sensitivity(0.0001);

        // let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/renderer_4v4n.vert.spv");
        // let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/renderer_4v4n.frag.spv");

        // Render pipeline...
        let t = TestLayoutEntry::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                        label: Some("renderer_v4n4_module"),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/renderer_v4n4.wgsl"))),
                    // &configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    //     label: Some("renderer_v4n4_module"),
                    //     source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/renderer_v4n4.wgsl"))),
                    //     flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
                    })
//                    &configuration.device.create_shader_module(&vertex_shader_src),
//                    &configuration.device.create_shader_module(&fragment_shader_src)
        );

        // Create bind groups for basic render pipeline and grass/rock textures. 
        let t_bindgroups = create_bind_groups(
                                &configuration.device, 
                                &t.layout_entries,
                                &vec![
                                    vec![&wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                            buffer: &camera.get_camera_uniform(&configuration.device),
                                            offset: 0,
                                            size: None,
                                    })], 
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
                                         vec![&wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                                 buffer: &camera.get_camera_uniform(&configuration.device),
                                                 offset: 0,
                                                 size: None,
                                         })], 
                                         vec![&wgpu::BindingResource::TextureView(&textures.get("slime").unwrap().view),
                                              &wgpu::BindingResource::Sampler(&textures.get("slime").unwrap().sampler),
                                              &wgpu::BindingResource::TextureView(&textures.get("slime2").unwrap().view),
                                              &wgpu::BindingResource::Sampler(&textures.get("slime2").unwrap().sampler)]
                                     ]
        );


        // The environment (mountains marching cubes). We need to disable shader validation because atomic counter are not supported yet.
        let mut mc_mountain = wgpu::include_spirv_raw!("../../shaders/spirv/mc_test.comp.spv");
        //mc_mountain.flags = wgpu::ShaderFlags::empty();

        let module = unsafe { &configuration.device.create_shader_module_spirv(&mc_mountain) };

        let mc = MarchingCubes::init(
            &configuration.device,
            &module,
            //++&configuration.device.create_shader_module_spirv(&mc_mountain),
            //&configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor { 
            //    label: Some("nojaa"), 
            //    source: wgpu::util::make_spirv(&include_bytes!("../../shaders/mc_test.comp")[..]),
            //    flags: wgpu::ShaderFlags::VALIDATION, //empty(),
            //}),
            //&configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test.comp.spv")),
            //&configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            //    label: Some("marching_cubes_test"),
            //    source: wgpu::ShaderSource::SpirV(Cow::Borrowed(ahhaa)),
            //    //source: wgpu::ShaderSource::SpirV(Cow::Borrowed(wgpu::include_spirv!("../../shaders/spirv/mc_test.comp.spv"))),
            //    flags: wgpu::ShaderFlags::VALIDATION,
            //    //flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
            //}),
            //&configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            //    label: Some("marching_cubes_test"),
            //    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/mc_test.wgsl"))),
            //    flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
            //}),
            false
        ); 

        // Create output buffer for "mountains", the output of mc.
        buffers.insert(
            "mc_output".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            &vec![0 as f32 ; 128*128*64*24],
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
            &buffers.get("mc_output").unwrap(),
            None,
        );

        // Add create bind groups to the mc_params.
        mc_params.bind_groups = Some(mc_bind_groups); 
        
        // GLSL, validation disabled.
        let mut slime_spirv = wgpu::include_spirv_raw!("../../shaders/spirv/mc_test_slime_noise3d_texture.comp.spv");
        //slime_spirv.flags = wgpu::ShaderFlags::empty();

        let module_slime = unsafe { &configuration.device.create_shader_module_spirv(&slime_spirv)};

        // The slime marching cubes.
        let mc_slime = MarchingCubes::init(
            &configuration.device,
            //&configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            //    label: Some("marching_cubes_silme_noide3d_test"),
            //    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/mc_test_slime_noise3d_texture.wgsl"))),
            //    flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
            //}),
            //&configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test_slime_noise3d_texture.comp.spv")),
            &module_slime,
            true
        );

        // Create output buffer for slime triangle mesh (the output of slime mc).
        buffers.insert(
            "mc_output_slime".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            //&vec![0 as f32 ; 128*128*64*24],
            &vec![0 as f32 ; 128*128*80*24],
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            None)
        );

        // Create parameters for "slime" marching cubes.
        let mut mc_params_slime = McParams::init(
                &configuration.device, 
                &cgmath::Vector4::<f32>::new(0.0,0.5,0.0,1.0),
                0.0,
                0.05
        );

        // Create density values buffer for slime.
        buffers.insert(
            "3dnoise_slime".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            //&vec![0 as f32 ; 64*2*64*16*4],
            &vec![0 as f32 ; 256*8*256],
            //&vec![0 as f32 ; 256*12*256],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );
        // Future usage1
        buffers.insert(
            "future_usage1_noise3d".to_string(), 
            buffer_from_data::<f32>(
            &configuration.device,
            &vec![0.3,0.3,0.3,0.3],
            wgpu::BufferUsages::COPY_DST |wgpu::BufferUsages::STORAGE,
            None)
        );

        // Create bind groups for slime.
        let mc_bind_groups_slime = mc_slime.create_bind_groups(
            &configuration.device,
            &mc_params_slime,
            &buffers.get("mc_output_slime").unwrap(),
            Some(&buffers.get("3dnoise_slime").unwrap())
        );

        // Add create bind groups to the mc_slime.
        mc_params_slime.bind_groups = Some(mc_bind_groups_slime); 

        // Create nouse 3d "texture".
        
        // GLSL, validation disabled.
        let mut shader_comp_3d_tex = wgpu::include_spirv_raw!("../../shaders/spirv/data3d_test.comp.spv");
        //shader_comp_3d_tex.flags = wgpu::ShaderFlags::empty();
                                   
        let module_comp3d = unsafe { &configuration.device.create_shader_module_spirv(&shader_comp_3d_tex)};

        let texture3_d = Custom3DTexture::init(
                &configuration.device,
                module_comp3d,
                //&configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                //    label: Some("texture3_d"),
                //    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/data3d_test.wgsl"))),
                //    flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
                //}),
        );

        // Create uniform buffer uvec3 for number of invocations.
        buffers.insert(
            "slime_invocations".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            //&vec![64,3,64],
            &vec![64,2,64],
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            None)
        );
        // Create uniform buffer uvec3 for area dimensions.
        buffers.insert(
            "slime_dimensions".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec![256,8,256],
            //&vec![256,12,256],
            //&vec![256,24,256],
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            None)
        );

        let slime_texture3d_bindgroups =
                create_bind_groups(
                    &configuration.device, 
                    &texture3_d.layout_entries,
                    &vec![
                        vec![&wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffers.get("slime_invocations").unwrap(),
                                offset: 0,
                                size: None,
                        }), 
                        &wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffers.get("slime_dimensions").unwrap(),
                                offset: 0,
                                size: None,
                        }), 
                        &wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffers.get("future_usage1_noise3d").unwrap(),
                                offset: 0,
                                size: None,
                        }), 
                        &wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffers.get("3dnoise_slime").unwrap(),
                                offset: 0,
                                size: None,
                        })]
                    ]
        );
        log::info!("Create mountain and first slime");

        // Perform both mountain and slime marching cubes.
        let mut encoder = configuration.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute encoder. Initial.") });

        mc.dispatch(&mc_params.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    128,
                    64
        ); 

        texture3_d.dispatch(&slime_texture3d_bindgroups,
                    &mut encoder,
                    64 * 2 * 64,
                    //64 * 3 * 64,
                    1,
                    1
        ); 

        log::info!("Third dispatch, slime");
        mc_slime.dispatch(&mc_params_slime.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    2,
                    //3,
                    64
        );

        configuration.queue.submit(Some(encoder.finish()));
        log::info!("Dispatch finished.");

        // The number of mountain vertices (from marching cubes).
        let k = to_vec::<u32>(&configuration.device,
                              &configuration.queue,
                              &mc_params.counter_buffer,
                              0 as wgpu::BufferAddress,
                              4 as wgpu::BufferAddress);

        // The number of initial slime vertices (from marching cubes).
        let k_slime = to_vec::<u32>(&configuration.device,
                                    &configuration.queue,
                                    &mc_params_slime.counter_buffer,
                                    0 as wgpu::BufferAddress,
                                    4 as wgpu::BufferAddress);

        log::info!("Application data initialized.");

        HelloApp {
            textures: textures,
            buffers: buffers,
            //two_triangles: two_triangles,
            //two_triangles_bind_group: bind_group,
            depth_texture: depth_texture,
            camera: camera,
            mc_slime: mc_slime,
            test_layout: t,
            bind: t_bindgroups,
            bind_slime: t_slime_bindgroups,
            draw_count_mc: k[0],
            draw_count_mc_slime: k_slime[0],
            mc_params_slime: mc_params_slime,
            slime_texture3d_bindgroups: slime_texture3d_bindgroups,
            custom_3d: texture3_d,
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

        // The mountain.
        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.bind,
             &self.test_layout.pipeline,
             &self.buffers.get("mc_output").unwrap(),
             0..self.draw_count_mc, 
             true
        );

        // The slime.
        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.bind_slime,
             &self.test_layout.pipeline,
             &self.buffers.get("mc_output_slime").unwrap(),
             0..self.draw_count_mc_slime, 
             false
        );

        queue.submit(Some(encoder.finish()));
    }

    fn input(&mut self, queue: &wgpu::Queue, input_cache: &InputCache) {
        // self.camera.update_from_input(&queue, &input_cache);
    }

    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, _new_size: winit::dpi::PhysicalSize<u32>) {
        self.depth_texture = JTexture::create_depth_texture(&device, &sc_desc, Some("depth-texture"));
        self.camera.resize(sc_desc.width as f32, sc_desc.height as f32);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, input: &InputCache) {

        let val = ((input.get_time() / 5000000) as f32) * 0.0015;

        //log::info!("val == {}", val);

        queue.write_buffer(
            &self.buffers.get("future_usage1_noise3d").unwrap(),
            0,
            bytemuck::cast_slice(&vec![val, 0.0, 0.0, 0.0])
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute encoder (noise/slime).") });

        self.camera.update_from_input(&queue, &input);

        // Create a new density scalar field for marching cubes slime.
        self.custom_3d.dispatch(&self.slime_texture3d_bindgroups,
                    &mut encoder,
                    64 * 2 * 64,
                    //64 * 3 * 64,
                    1,
                    1
        );
        
        // Create slime.
        self.mc_params_slime.reset_counter(&queue);
        self.mc_params_slime.update_params(
            &queue,
            &Some(cgmath::Vector4::<f32>::new(0.0,0.5,0.0,1.0)),
            &Some(0.0),
            &None,
            &Some(0.05),
        ); 
        //let mut mc_params_slime = McParams::init(
        //        &configuration.device, 
        //        cgmath::Vector4::<f32>::new(0.0,0.5,0.0,1.0),
        //        0.0,
        //        0.05

        self.mc_slime.dispatch(&self.mc_params_slime.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    2,
                    //3,
                    64
        );

        queue.submit(Some(encoder.finish()));

        let k_slime = to_vec::<u32>(&device,
                                    &queue,
                                    &self.mc_params_slime.counter_buffer,
                                    0 as wgpu::BufferAddress,
                                    4 as wgpu::BufferAddress);

        self.draw_count_mc_slime = k_slime[0];
        //log::info!("k_slime[0] == {}", k_slime[0]);
    }
}

fn main() {
    
    ws::run_loop::<HelloApp, BasicLoop, MyFeatures>(); 
    println!("Finished...");
}
