use cgmath::{Vector3};
use std::collections::HashMap;
use jaankaup_core::buffer::*;
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
use jaankaup_core::compute::*;
use geometry::aabb::BBox;
use debug_shaders::aabb_shader::AABB_pipeline;
use jaankaup_core::render_pipelines::{
    create_bind_groups,
    draw
};
use jaankaup_core::mc::*;
use render_shaders::Render_vvvvnnnn;

// Redefine needed features for this application.
struct Debug_Features {}
impl WGPUFeatures for Debug_Features {
}

// The fmm application.
struct Debug_App {
    depth_texture: JTexture,
    camera: Camera,
    aabb: AABB_pipeline,
    buffers: HashMap<String, wgpu::Buffer>,
    aabbs: Vec<BBox>,
    histogram: Histogram,
    bind_groups: Vec<wgpu::BindGroup>,
    mc_distance: MarchingCubes,
    draw_count_mc_distance: u32,
    mc_params_distance: McParams,
    render_pipeline: Render_vvvvnnnn,
    render_bind_groups: Vec<wgpu::BindGroup>,
}

impl Debug_App {

}

impl Application for Debug_App {

    /// Initialize fmm application.
    fn init(configuration: &WGPUConfiguration) -> Self {

        // Initialize camera.
        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);
        camera.set_movement_sensitivity(0.1);

        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvvnnnn_camera.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvvnnnn_camera.frag.spv");
        let render_pipeline = Render_vvvvnnnn::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src)
        );
        let render_bind_groups = render_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );

        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
        let mut aabbs: Vec<BBox> = Vec::new();

        aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(0.3,0.5,0.0), &Vector3::<f32>::new(2.3,1.5,1.0)));
        //aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(1.3,0.5,-1.0), &Vector3::<f32>::new(1.3,2.5,1.0)));
        //aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(3.3,0.5,-1.0), &Vector3::<f32>::new(7.3,2.5,1.0)));

        // Create the depth texture for fmm application.
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("debug depth texture")
        );

        // Histogram.
        let histogram = Histogram::init(&configuration.device, 1, 0); 

        // Create density values buffer for slime.
        buffers.insert(
            "distance_data".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            &vec![5000.0 as f32 ; 64*64*64*64],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );

        // Create buffers for aabbs.
        buffers.insert(
            "aabbs".to_string(),
            buffer_from_data::<BBox>(
            &configuration.device,
            &aabbs,
            wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        // Numbers of aabbs.
        buffers.insert(
            "num_of_aabbs".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &[aabbs.len() as u32],
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
            None)
        );

        // Numbers of aabbs.
        buffers.insert(
            "dimensions".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec![256, 256, 256],
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
            None)
        );

        buffers.insert(
            "mc_output_distance".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &vec![0 as f32 ; 256*128*128*24],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        let aabb = AABB_pipeline::init(&configuration.device);

        let bind_groups = create_bind_groups(
                                &configuration.device, 
                                &aabb.get_bind_group_layout_entries(),
                                &vec![
                                    vec![
                                        &buffers.get("dimensions").unwrap().as_entire_binding(),
                                        &buffers.get("num_of_aabbs").unwrap().as_entire_binding(),
                                        &buffers.get("aabbs").unwrap().as_entire_binding(),
                                        &buffers.get("distance_data").unwrap().as_entire_binding(),
                                    ], 
                                ]
        );

        // Mc for visualizing distance fields.
        let mc_distance = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_test_slime_noise3d_texture.comp.spv")),
            true
        );

        // Create parameters for "slime" marching cubes.
        let mut mc_params_distance = McParams::init(
                &configuration.device, 
                &cgmath::Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0),
                0.0,
                0.05
        );

        // Create bind groups for distance field.
        let mc_bind_groups_distance = mc_distance.create_bind_groups(
            &configuration.device,
            &mc_params_distance,
            &buffers.get("mc_output_distance").unwrap(),
            Some(&buffers.get("distance_data").unwrap())
        );

        // Add create bind groups to the mc_slime.
        mc_params_distance.bind_groups = Some(mc_bind_groups_distance); 

        // Launch distance calculation and mc.
        let mut encoder = configuration.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Juuu") });

        aabb.dispatch(&bind_groups,
                    &mut encoder,
                    64 * 64 * 64,
                    1,
                    1
        ); 

        mc_distance.dispatch(&mc_params_distance.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    64,
                    64,
                    64
        ); 

        configuration.queue.submit(Some(encoder.finish()));


        let k =  pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &mc_params_distance.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));
        ////let distance_data =  pollster::block_on(to_vec::<f32>(&configuration.device,
        ////                                      &configuration.queue,
        ////                                      &buffers.get("distance_data").unwrap(),
        ////                                      0 as wgpu::BufferAddress,
        ////                                      64*64*64*64*4 as wgpu::BufferAddress));
        //#[cfg(not(target_arch = "wasm32"))]
        println!("Mc counter distance == {}", k[0]);

        ////for i in 0..distance_data.len() {
        ////    println!("{} :: {}", i, distance_data[i]);
        ////}

        let draw_count_mc_distance = k[0];



        Self {
            depth_texture,
            camera,
            aabb,
            buffers,
            aabbs,
            histogram,
            bind_groups,
            mc_distance,
            draw_count_mc_distance,
            mc_params_distance,
            render_pipeline,
            render_bind_groups,
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
                //log::info!("FAILED");
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
             &self.render_bind_groups,
             &self.render_pipeline.get_pipeline(),
             &self.buffers.get("mc_output_distance").unwrap(),
             0..self.draw_count_mc_distance,
             true
        );

        // // self.two_triangles.draw(&mut encoder, &frame, &self.depth_texture, &self.two_triangles_bind_group, true);

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

    }
}

fn main() {
    ws::run_loop::<Debug_App, BasicLoop, Debug_Features>(); 
}
