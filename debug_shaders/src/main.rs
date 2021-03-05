use rand::prelude::*;
use cgmath::{prelude::*,Vector3,Vector4};
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
use geometry::aabb::{BBox,BBox4};
use debug_shaders::aabb_shader::AABB_pipeline;
use debug_shaders::font::Font_pipeline;
use jaankaup_core::render_pipelines::{
    create_bind_groups,
    draw,
    check_correspondence,
};
use jaankaup_core::mc::*;
use render_shaders::{Render_vvvvnnnn, Render_vvvc};

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
    aabbs: Vec<BBox4>,
    histogram: Histogram,
    bind_groups: Vec<wgpu::BindGroup>,
    mc_distance: MarchingCubes,
    draw_count_mc_distance: u32,
    mc_params_distance: McParams,
    render_pipeline: Render_vvvvnnnn,
    render_bind_groups: Vec<wgpu::BindGroup>,
    font_pipeline: Font_pipeline,
    font_bind_groups: Vec<wgpu::BindGroup>,
    render_vvvc_pipeline: Render_vvvc,
    render_vvvc_bind_groups: Vec<wgpu::BindGroup>,
    vvvc_draw_count: u32,
}

struct JooAABB {
    min: Vector4<f32>,
    max: Vector4<f32>,
}

pub fn bezier_1c(n: u32, 
                c0: Vector3<f32>,
                c1: Vector3<f32>,
                c2: Vector3<f32>
                ) -> Vec<Vector3<f32>> {
    assert!(n > 1, "n < 2");
    let mut result: Vec<Vector3<f32>> = Vec::new();
    for i in 0..n {
        let t = i as f32 / ((n-1) as f32);
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let x =   c0.x  * mt2 + c1.x * 2.0 * mt*t + c2.x * t2;
        let y =   c0.y  * mt2 + c1.y * 2.0 * mt*t + c2.y * t2;
        let z =   c0.z  * mt2 + c1.z * 2.0 * mt*t + c2.z * t2;
        result.push(Vector3::<f32>::new(x, y, z));
    }
    result
}

pub fn line(n: u32,
            c0: Vector3<f32>,
            c1: Vector3<f32>,
            ) -> Vec<Vector3<f32>> {
    assert!(n > 0, "n == 0");
    let mut result: Vec<Vector3<f32>> = Vec::new();
    for i in 0..n {

        let t = i as f32 / (n as f32);
        //let mt = 1.0 - t;

        let dist = c1.distance(c0); 
        let norm = (c1 - c0).normalize();

        let point = c0 + norm * (dist * t);

        result.push(point);
    }
    result
}

// (1.0, 1.0, 1,0) , (23.0, 23.0, 23.0, 5.0) , (1.0, 1.0, 15.0) , (23.0, 23.0, 20.0)
pub fn bezier3D(n: u32,
                c0: Vector3<f32>,
                c1: Vector3<f32>,
                c2: Vector3<f32>,
                c3: Vector3<f32>,
                ) -> Vec<Vector3<f32>> {
    assert!(n > 0, "n == 0");
    let mut result: Vec<Vector3<f32>> = Vec::new();
    for i in 0..n {
        let t = i as f32 / (n as f32);
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let x = c0.x  * mt3 + c1.x * 3.0 * mt2*t + c2.x * 3.0 * mt*t2 + c3.x * t3;
        let y = c0.y  * mt3 + c1.y * 3.0 * mt2*t + c2.y * 3.0 * mt*t2 + c3.y * t3;
        let z = c0.z  * mt3 + c1.z * 3.0 * mt2*t + c2.z * 3.0 * mt*t2 + c3.z * t3;
        result.push(Vector3::<f32>::new(x, y, z));
    }
    result
}

impl Application for Debug_App {

    /// Initialize fmm application.
    fn init(configuration: &WGPUConfiguration) -> Self {

        // Initialize camera.
        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);
        camera.set_movement_sensitivity(0.05);

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

        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvc_camera.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvc_camera.frag.spv");

        let render_vvvc_pipeline = Render_vvvc::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src)
        );
        let render_vvvc_bind_groups = render_vvvc_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );

        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
        let mut aabbs: Vec<BBox4> = Vec::new();

        //aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(12.0,3.0,10.0), &Vector3::<f32>::new(13.0,20.0, 11.0)).convert_aabb_to_aabb4());
        //aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(10.0,3.0,10.0), &Vector3::<f32>::new(15.0,4.0, 11.0)).convert_aabb_to_aabb4());
        //aabbs.push(BBox::create_from_line(&Vector3::<f32>::new(10.0,19.0,10.0), &Vector3::<f32>::new(11.0,20.0, 11.0)).convert_aabb_to_aabb4());
        let n0 = 10;
        let mut base_vecs = bezier_1c(n0, 
                            Vector3::<f32>::new(0.5, 0.9, 0.1),
                            Vector3::<f32>::new(0.4, 0.85, 0.1),
                            Vector3::<f32>::new(0.3, 0.8, 0.1),
        );

        let n1 = 20;
        let base_vecs2 = line(n1, Vector3::<f32>::new(0.5, 0.1, 0.1), Vector3::<f32>::new(0.5, 0.9, 0.1));

        let n2 = 10;
        let base_vecs3 = line(n2, Vector3::<f32>::new(0.3, 0.1, 0.1), Vector3::<f32>::new(0.7, 0.1, 0.1));


        base_vecs.extend(base_vecs2);
        base_vecs.extend(base_vecs3);

        // Create the depth texture for fmm application.
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("debug depth texture")
        );

        for i in 0..base_vecs.len() {
            let a = base_vecs[i];
            println!("{:?}", base_vecs[i]);
            let b = base_vecs[i]
                + Vector3::<f32>::new(
                    base_vecs[i].x + 1.0,
                    base_vecs[i].y + 1.0,
                    base_vecs[i].z + 1.0,
            );
            aabbs.push(BBox::create_from_line(&a, &b).convert_aabb_to_aabb4());
        }


        // Create density values buffer for slime.
        buffers.insert(
            "distance_data".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            &vec![5000.0 as f32 ; 6*6*6*64],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );

        // Create buffers for aabbs.
        buffers.insert(
            "aabbs".to_string(),
            buffer_from_data::<BBox4>(
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

        buffers.insert(
            "dimensions".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec![24, 24, 24],
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
            None)
        );

        buffers.insert(
            "dimensions_font".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec![64, 64, 64],
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
            None)
        );

        buffers.insert(
            "mc_output_distance".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &vec![0 as f32 ; 128*128*128*24],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        buffers.insert(
            "font_output".to_string(),
            buffer_from_data::<f32>(
            &configuration.device,
            // gl_Position     |    point_pos
            &vec![0 as f32 ; 128*128*128*32],
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

        // Histogram.
        let histogram = Histogram::init(&configuration.device, 1, 0); 

        let font_pipeline = Font_pipeline::init(&configuration.device);

        check_correspondence(
            &font_pipeline.get_bind_group_layout_entries(),
            &vec![
                vec![
                    &buffers.get("dimensions_font").unwrap().as_entire_binding(),
                    &histogram.get_histogram().as_entire_binding(),
                    &buffers.get("font_output").unwrap().as_entire_binding(),
                ], 
            ]
        );


        let font_bind_groups = create_bind_groups(
                                &configuration.device, 
                                &font_pipeline.get_bind_group_layout_entries(),
                                &vec![
                                    vec![
                                        &buffers.get("dimensions_font").unwrap().as_entire_binding(),
                                        &histogram.get_histogram().as_entire_binding(),
                                        &buffers.get("font_output").unwrap().as_entire_binding(),
                                    ], 
                                ]
        );

        // Mc for visualizing distance fields.
        let mc_distance = MarchingCubes::init(
            &configuration.device,
            &configuration.device.create_shader_module(&wgpu::include_spirv!("../../shaders/spirv/mc_distance_field.comp.spv")),
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
                    6 * 6 * 6,
                    1,
                    1
        );

        mc_distance.dispatch(&mc_params_distance.bind_groups.as_ref().unwrap(),
                    &mut encoder,
                    6,
                    6,
                    6
        ); 

        font_pipeline.dispatch(&font_bind_groups,
                    &mut encoder,
                    16 * 16 * 16,
                    1,
                    1
        ); 

        configuration.queue.submit(Some(encoder.finish()));

        let k =  pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &mc_params_distance.counter_buffer,
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));
        let distance_data =  pollster::block_on(to_vec::<f32>(&configuration.device,
                                              &configuration.queue,
                                              &buffers.get("distance_data").unwrap(),
                                              0 as wgpu::BufferAddress,
                                              64*6*6*6*4 as wgpu::BufferAddress));
        //for i in 0..distance_data.len() {
        //for i in 0..distance_data.len() {
        for i in 0..100 {
            println!("{} :: {}", i, distance_data[i]);
        }

        //#[cfg(not(target_arch = "wasm32"))]
        println!("Mc counter distance == {}", k[0]);


        let draw_count_mc_distance = k[0];

        let font_k = pollster::block_on(to_vec::<u32>(&configuration.device,
                                              &configuration.queue,
                                              &histogram.get_histogram(),
                                              0 as wgpu::BufferAddress,
                                              4 as wgpu::BufferAddress));

        println!("Histogram == {}", font_k[0]);

        let vvvc_draw_count = font_k[0];

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
            font_pipeline,
            font_bind_groups,
            render_vvvc_pipeline,
            render_vvvc_bind_groups,
            vvvc_draw_count,
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

        // draw(&mut encoder,
        //      &frame,
        //      &self.depth_texture,
        //      &self.render_bind_groups,
        //      &self.render_pipeline.get_pipeline(),
        //      &self.buffers.get("mc_output_distance").unwrap(),
        //      0..self.draw_count_mc_distance,
        //      true
        // );

        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.render_vvvc_bind_groups,
             &self.render_vvvc_pipeline.get_pipeline(),
             &self.buffers.get("font_output").unwrap(),
             0..self.vvvc_draw_count,
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
