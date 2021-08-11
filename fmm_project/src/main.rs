//#[macro_use]
use jaankaup_core::impl_convert;
use jaankaup_core::misc::Convert2Vec;
//extern crate jaankaup_core;//::mics::impl_convert;
//extern crate jaankaup_core::mics::impl_convert;

use std::mem;
use std::borrow::Cow;
use rand::prelude::*;
pub use winit::event::VirtualKeyCode as Key;
use jaankaup_core::wgpu;
use render_shaders::{Render_vvvc, Render_vvvvnnnn};
use jaankaup_core::buffer::{buffer_from_data, to_vec};
use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop,
};
use jaankaup_core::two_triangles::TwoTriangles;

use jaankaup_core::render_pipelines::{
    create_bind_group_layouts,
    draw,
    create_bind_groups
};

use jaankaup_core::compute::Histogram;
use jaankaup_core::texture::Texture as JTexture;
use jaankaup_core::camera::{Camera};
use jaankaup_core::input::InputCache;
use jaankaup_core::misc::OutputVertex;
use index_tables::create_hash_table;
use geometry::aabb::{BBox, Triangle, Triangle_vvvvnnnn};
use model_loader::load_triangles_from_obj;
use bytemuck::{Pod, Zeroable};

//static DEBUG_BUFFER_SIZE: u32 = 33554432;
//TODO: rename. This is not the buffer size. It's the number on poinst, and the number of line
//triangle points.
const DEBUG_BUFFER_SIZE: u32   = 1024000; //4194300; // 1048575; //33554416;
const DEBUG_BUFFER_OFFSET: u32 = 1024000; // 2097151 / 2 ~= 1048574
//const BLOCK_DIMENSIONS: [u32; 3] = [32, 32, 32];
const BLOCK_DIMENSIONS: [u32; 3] = [4, 8, 4];
const TIME_STAMP_COUNT: u32 = 2;

const FAR: u32 =  3;
const BAND: u32 = 1;
const BAND_NEW: u32 = 1;
const KNOWN: u32 = 0;
//const KNOWN_NEW: u32 = 1;
//const NEW_BAND: u32 = 2;
// TODO: add Queries to jaankaup_core.
  
struct QuerySets {
    timestamp: wgpu::QuerySet,
    timestamp_period: f32,
    //pipeline_statistics: wgpu::QuerySet,
    query_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
struct TimestampData {
    start: u64,
    end: u64,
}

impl_convert!{TimestampData}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
struct QueryData {
    timestamps: [TimestampData; TIME_STAMP_COUNT as usize],
    // pipeline_queries: [u64; MIP_PASS_COUNT as usize],
}

impl_convert!{QueryData}

// Redefine needed features for this application.
struct FMM_Features {}
impl WGPUFeatures for FMM_Features {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::TIMESTAMP_QUERY
        // wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::SPIRV_SHADER_PASSTHROUGH |
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        //wgpu::Features::ALL_NATIVE
    }
    fn required_limits() -> wgpu::Limits {
        let mut limits = wgpu::Limits::default();
        limits.max_storage_buffers_per_shader_stage = 8;
        //limits.max_storage_buffer_binding_size = 3193724;
        limits
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FMM_Block {
    //base_coord: [u32 ; 3],
    index: u32,
    band_points_count: u32,
}

unsafe impl bytemuck::Zeroable for FMM_Block {}
unsafe impl bytemuck::Pod for FMM_Block {}

#[repr(C)]
#[derive(Copy, Clone)]
struct FMM_Node {
    value: f32,
    tag: u32,
}

unsafe impl bytemuck::Zeroable for FMM_Node {}
unsafe impl bytemuck::Pod for FMM_Node {}

#[repr(C)]
#[derive(Copy, Clone)]
struct FMM_Attributes {
    global_dimensions: [u32 ; 3],
    offset_hash_table_size: u32,
    current_block: [u32;3], // check alignment
    vec_to_offset_table_size: u32,
}

unsafe impl bytemuck::Zeroable for FMM_Attributes {}
unsafe impl bytemuck::Pod for FMM_Attributes {}

// The fmm application.
struct FMM_App {
    textures: HashMap<String, JTexture>,
    depth_texture: JTexture,
    camera: Camera,
    buffers: HashMap<String, wgpu::Buffer>,
    render_vvvc_point_pipeline: Render_vvvc, 
    render_vvvc_point_bind_groups: Vec<wgpu::BindGroup>,
    render_vvvc_triangle_pipeline: Render_vvvc, 
    render_vvvc_triangle_bind_groups: Vec<wgpu::BindGroup>,
    fmm_debug_pipeline: FMM_debug_pipeline,
    fmm_debug_bind_groups: Vec<wgpu::BindGroup>,
    histogram: Histogram, 
    debug_point_count: u32,
    debug_triangle_draw_count: u32,
    fmm_data_generator: FMM_data_generator_debug_pipeline,
    fmm_data_generator_bind_groups: Vec<wgpu::BindGroup>,
    render_vvvvnnnn_pipeline: Render_vvvvnnnn,
    render_vvvvnnnn_bind_groups: Vec<wgpu::BindGroup>,
    show_mesh: bool,
    current_block: [f32;3], 
    fmm_attributes: FMM_Attributes,
    current_global_dimensions: [f32;3], 
    triangle_count: u32,
    update_data_generator: u32,
    triangle_data: Vec<Triangle_vvvvnnnn>,
    triangle_index: f32,
    show_whole_mesh: u32,
    changed: u32,
    data_loaded: bool,
    query_sets: Option<QuerySets>,
    screen: TwoTriangles,
    sphere_tracer_pipeline: SphereTracerPipeline, 
    sphere_tracer_bind_groups: Vec<wgpu::BindGroup>,
}

impl FMM_App {

}

impl Application for FMM_App {

    /// Initialize fmm application.
    fn init(configuration: &WGPUConfiguration) -> Self {

        // Create queries for time stamps.
        let query_sets = if configuration.device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY) {

            let timestamp = configuration.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("timestamppi"),
                count: TIME_STAMP_COUNT * 2, // count * numb(begining/end)
                ty: wgpu::QueryType::Timestamp,
            });
            let timestamp_period = configuration.queue.get_timestamp_period();

            println!("mem::size_of::<QueryData>() == {}", mem::size_of::<QueryData>());
            // TODO: Try to implement buffer reading without 'wgpu::BufferUsages::COPY_SRC'
            let query_buffer = configuration.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query buffer"),
                size: mem::size_of::<QueryData>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            Some(QuerySets {
                timestamp,
                timestamp_period,
                query_buffer,
            })
        } else {
            None
        };

        let mut textures: HashMap<String, JTexture> = HashMap::new();
        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();

        let changed = 0;
        let data_loaded = false;

        // Create the depth texture for fmm application.
        let depth_texture = JTexture::create_depth_texture(
            &configuration.device,
            &configuration.sc_desc,
            Some("fmm depth texture")
        ); 

        // Create the index hash table for local indexing in GPU (includes ghost region).
        let (offset_hash_table, vec_to_offset_table, ivec_offset_hash_table) =
            create_hash_table(4, 4, 4, BLOCK_DIMENSIONS[0], BLOCK_DIMENSIONS[1], BLOCK_DIMENSIONS[2]);

        let current_block: [f32; 3] = [0.0,0.0,0.0];
        let current_global_dimensions: [f32; 3] = [BLOCK_DIMENSIONS[0] as f32, BLOCK_DIMENSIONS[1] as f32, BLOCK_DIMENSIONS[2] as f32];

        let fmm_attributes = FMM_Attributes{ global_dimensions: BLOCK_DIMENSIONS,
                                             offset_hash_table_size: ivec_offset_hash_table.len() as u32,
                                             current_block: [0,0,0],
                                             vec_to_offset_table_size: vec_to_offset_table.len() as u32,
        };
        let update_data_generator = 0;

        let sphere_tracer_texture =
            JTexture::create_texture2d(&configuration.device, &configuration.sc_desc, 1, 256, 256);

        textures.insert("sphere_tracer_texture".to_string(), sphere_tracer_texture);

        // Create the sphere tracer screen.
        let screen = TwoTriangles::init(&configuration.device, &configuration.sc_desc);
        let screen_group = TwoTriangles::create_bind_group(
            &configuration.device,
            &textures.get("sphere_tracer_texture").unwrap() 
        );

        buffers.insert(
            "sphere_tracer_output".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec![0; 256*256],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        buffers.insert(
            "index_hash_table".to_string(),
            buffer_from_data::<[i32; 4]>(
            &configuration.device,
            &ivec_offset_hash_table,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        buffers.insert(
            "vec_to_offset".to_string(),
            buffer_from_data::<u32>(
            &configuration.device,
            &vec_to_offset_table,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        println!("Creating fmm attributes");

        buffers.insert(
            "fmm_attributes".to_string(),
            buffer_from_data::<FMM_Attributes>(
            &configuration.device,
            &[fmm_attributes],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            None)
        );

        let (_, triangle_data, aabb): (Vec<Triangle>, Vec<Triangle_vvvvnnnn>, BBox) =
            load_triangles_from_obj("assets/models/wood.obj", 7.0, [20.0, 0.0, 26.0], None).unwrap();
            //load_triangles_from_obj("assets/models/wood.obj", 1.0, [5.0, -5.0, 18.0], Some(1)).unwrap();

        let triangle_count: u32 = triangle_data.len() as u32;

        println!("WOOD vertex count (vvvv) = {}", triangle_data.len());
        println!("WOOD aabb = {:?}", aabb);

        let triangle_index: f32 = 0.0;
        let show_whole_mesh = 0;
        buffers.insert(
            "wood".to_string(),
            buffer_from_data::<Triangle_vvvvnnnn>(
            &configuration.device,
            &triangle_data,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        buffers.insert(
            "wood_single".to_string(),
            buffer_from_data::<Triangle_vvvvnnnn>(
            &configuration.device,
            &[triangle_data[0]],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        // Initialize camera for fmm application.
        let mut camera = Camera::new(configuration.size.width as f32, configuration.size.height as f32);
        camera.set_movement_sensitivity(0.01);
        //camera.set_rotation_sensitivity(2.0);
        camera.set_rotation_sensitivity(0.2);


        // Create buffers for fmm alogrithm.
        create_buffers(&configuration.device,
                       BLOCK_DIMENSIONS, // block_dimension: [u32 ; 3],
                       [4, 4, 4], //local_dimension: [u32 ; 3],
                       DEBUG_BUFFER_SIZE, // 3193724
                       &mut buffers
        );

        // The point pipeline.
        let render_vvvc_point_pipeline = Render_vvvc::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                        label: Some("renderer_v3c1_module1"),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/renderer_v3c1.wgsl"))),
                        //flags: wgpu::ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
                    }),
                    wgpu::PrimitiveTopology::PointList
        );
        let render_vvvc_point_bind_groups = render_vvvc_point_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );

        // The triangle pipeline.
        let render_vvvc_triangle_pipeline = Render_vvvc::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                        label: Some("renderer_v3c1_module2"),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/renderer_v3c1.wgsl"))),
                    }),
                    wgpu::PrimitiveTopology::TriangleList
        );
        let render_vvvc_triangle_bind_groups = render_vvvc_triangle_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );
        let render_vvvvnnnn_pipeline = Render_vvvvnnnn::init(
                &configuration.device,
                &configuration.sc_desc,
                &configuration.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("renderer_v4n4_module"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders_wgsl/renderer_v4n4_plain.wgsl"))),
                })
        );
        let render_vvvvnnnn_bind_groups = render_vvvvnnnn_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );

        let mut debug_point_count = 2; // WHY?
        let mut debug_triangle_draw_count = DEBUG_BUFFER_SIZE;

        // Create histogram for fmm debug.
        let mut histogram = Histogram::init(&configuration.device, &vec![0, 2]); 

        let fmm_debug_pipeline = FMM_debug_pipeline::init(&configuration.device);

        let fmm_debug_bind_groups =
                create_bind_groups(
                    &configuration.device, 
                    &fmm_debug_pipeline.get_bind_group_layout_entries(),
                    &vec![
                        vec![
                            &camera.get_camera_uniform(&configuration.device).as_entire_binding(),
                            &buffers.get("prefix_sum_temp").unwrap().as_entire_binding(),
                            &buffers.get("debug_points_output").unwrap().as_entire_binding(),
                            &buffers.get("fmm_nodes").unwrap().as_entire_binding(),
                            &buffers.get("fmm_blocks").unwrap().as_entire_binding(),
                            &histogram.get_histogram().as_entire_binding(),
                            &buffers.get("index_hash_table").unwrap().as_entire_binding(),
                            &buffers.get("vec_to_offset").unwrap().as_entire_binding(),
                            &buffers.get("fmm_attributes").unwrap().as_entire_binding(),
                        ], 
                    ]
        );
        let fmm_data_generator = FMM_data_generator_debug_pipeline::init(&configuration.device);
        let fmm_data_generator_bind_groups =
                create_bind_groups(
                    &configuration.device, 
                    &fmm_data_generator.get_bind_group_layout_entries(),
                    &vec![
                        vec![
                            &camera.get_camera_uniform(&configuration.device).as_entire_binding(),
                            &histogram.get_histogram().as_entire_binding(),
                            &buffers.get("debug_points_output").unwrap().as_entire_binding(),
                            &buffers.get("fmm_nodes").unwrap().as_entire_binding(),
                            &buffers.get("wood").unwrap().as_entire_binding(),
                            &buffers.get("index_hash_table").unwrap().as_entire_binding(),
                            &buffers.get("vec_to_offset").unwrap().as_entire_binding(),
                            &buffers.get("fmm_attributes").unwrap().as_entire_binding(),
                            &buffers.get("fmm_data_gen_params").unwrap().as_entire_binding(),
                        ], 
                    ]
        );

        println!("Creating Sphere tracer");
        let sphere_tracer_pipeline = SphereTracerPipeline::init(&configuration.device); 
        let sphere_tracer_bind_groups = 
                create_bind_groups(
                    &configuration.device, 
                    &sphere_tracer_pipeline.get_bind_group_layout_entries(),
                    &vec![
                        vec![
                            &camera.get_ray_camera_uniform(&configuration.device).as_entire_binding(),
                            &buffers.get("fmm_nodes").unwrap().as_entire_binding(),
                            &buffers.get("index_hash_table").unwrap().as_entire_binding(),
                            &buffers.get("vec_to_offset").unwrap().as_entire_binding(),
                            &buffers.get("fmm_attributes").unwrap().as_entire_binding(),
                            &buffers.get("sphere_tracer_output").unwrap().as_entire_binding(),
                            //&wgpu::BindingResource::TextureView(&textures.get("sphere_tracer_texture").unwrap().view),
                        ],
                    ]
        );
                
        println!("Creating Sphere tracer :: OK");

        let show_mesh = false;

        Self {
            textures,
            depth_texture,
            camera,
            buffers,
            render_vvvc_point_pipeline, 
            render_vvvc_point_bind_groups,
            render_vvvc_triangle_pipeline, 
            render_vvvc_triangle_bind_groups,
            fmm_debug_pipeline,
            fmm_debug_bind_groups,
            histogram,
            debug_point_count,
            debug_triangle_draw_count,
            fmm_data_generator,
            fmm_data_generator_bind_groups,
            render_vvvvnnnn_pipeline,
            render_vvvvnnnn_bind_groups,
            show_mesh,
            current_block,
            fmm_attributes,
            current_global_dimensions,
            triangle_count,
            update_data_generator,
            triangle_data,
            triangle_index,
            show_whole_mesh,
            changed,
            data_loaded,
            query_sets,
            screen,
            sphere_tracer_pipeline,
            sphere_tracer_bind_groups,
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
                *swap_chain = device.create_swap_chain(surface, sc_desc);
                swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture").output
            },
        };

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
        });
        let mut clear = true;

        if self.show_mesh {
            draw(&mut encoder,
                 &frame,
                 &self.depth_texture,
                 &self.render_vvvvnnnn_bind_groups,
                 &self.render_vvvvnnnn_pipeline.get_pipeline(),
                 &self.buffers.get("wood").unwrap(),
                 //0..300,
                 //0..2036*3,
                 0..self.triangle_count * 3,
                 clear
            );
            clear = false;
        }

        //++ draw(&mut encoder,
        //++      &frame,
        //++      &self.depth_texture,
        //++      &self.render_vvvvnnnn_bind_groups,
        //++      &self.render_vvvvnnnn_pipeline.get_pipeline(),
        //++      &self.buffers.get("wood_single").unwrap(),
        //++      //0..300,
        //++      //0..2036*3,
        //++      0..3,
        //++      clear
        //++ );
        //++ if clear { clear = false; }
        
        if self.debug_point_count > 0 {
           draw(&mut encoder,
                &frame,
                &self.depth_texture,
                &self.render_vvvc_point_bind_groups,
                &self.render_vvvc_point_pipeline.get_pipeline(),
                &self.buffers.get("debug_points_output").unwrap(),
                0..self.debug_point_count,
                //2..self.debug_point_count,
                //3..3000,
                clear
           );
           clear = false;
        }


        if self.debug_triangle_draw_count > DEBUG_BUFFER_SIZE {
           draw(&mut encoder,
                &frame,
                &self.depth_texture,
                &self.render_vvvc_triangle_bind_groups,
                &self.render_vvvc_triangle_pipeline.get_pipeline(),
                &self.buffers.get("debug_points_output").unwrap(),
                DEBUG_BUFFER_SIZE..self.debug_triangle_draw_count,
                clear
           );
        }
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

        // Reset debug counters.
        self.histogram.set_values_cpu_version(&queue, &vec![0, DEBUG_BUFFER_OFFSET]);

        // Get the keyboard state (camera movement).
        let space_pressed = input.key_state(&Key::Space);
        if !space_pressed.is_none() {
            self.update_data_generator = (self.update_data_generator + 1) % 10;
            if self.update_data_generator < 5 { self.show_mesh = true; }
            else { self.show_mesh = false; }
        }

        queue.write_buffer(
            &self.buffers.get("wood_single").unwrap(),
            0,
            bytemuck::cast_slice(&[self.triangle_data[self.triangle_index as usize]])
        );

        let time_offset = input.get_time_delta() as f32 / 150000000.0;

        // Global dimensions.
          
        let key1_pressed = input.key_state(&Key::Key1);
        let key2_pressed = input.key_state(&Key::Key2);
        let key3_pressed = input.key_state(&Key::Key3);
        let key4_pressed = input.key_state(&Key::Key4);
        let key5_pressed = input.key_state(&Key::Key5);
        let key6_pressed = input.key_state(&Key::Key6);

        let mut global_dimensions = self.current_global_dimensions;

        let mut pressed: bool = false;

        if !key1_pressed.is_none() {global_dimensions = [global_dimensions[0] - time_offset, global_dimensions[1], global_dimensions[2]]; pressed = true; }
        if !key2_pressed.is_none() {global_dimensions = [global_dimensions[0] + time_offset, global_dimensions[1], global_dimensions[2]];  pressed = true;}
        if !key3_pressed.is_none() {global_dimensions = [global_dimensions[0] , global_dimensions[1] - time_offset, global_dimensions[2]];  pressed = true;}
        if !key4_pressed.is_none() {global_dimensions = [global_dimensions[0] , global_dimensions[1] + time_offset, global_dimensions[2]];  pressed = true;}
        if !key5_pressed.is_none() {global_dimensions = [global_dimensions[0] , global_dimensions[1], global_dimensions[2] - time_offset];  pressed = true;}
        if !key6_pressed.is_none() {global_dimensions = [global_dimensions[0] , global_dimensions[1], global_dimensions[2] + time_offset];  pressed = true;}

        if global_dimensions[0] > 1.0 && global_dimensions[1] > 1.0 &&  global_dimensions[2] > 1.0 && pressed {

                self.fmm_attributes.global_dimensions = [global_dimensions[0] as u32, global_dimensions[1] as u32, global_dimensions[2] as u32];

                self.current_global_dimensions = global_dimensions; 

                if self.fmm_attributes.current_block[0] >= self.fmm_attributes.global_dimensions[0] { 
                    self.current_block[0] = self.fmm_attributes.global_dimensions[0] as f32 - 0.5;
                    self.fmm_attributes.current_block[0] = self.fmm_attributes.global_dimensions[0]-1;
                }
                if self.fmm_attributes.current_block[1] >= self.fmm_attributes.global_dimensions[1] { 
                    self.current_block[1] = self.fmm_attributes.global_dimensions[1] as f32 - 0.5;
                    self.fmm_attributes.current_block[1] = self.fmm_attributes.global_dimensions[1]-1;
                }
                if self.fmm_attributes.current_block[2] >= self.fmm_attributes.global_dimensions[2] { 
                    self.current_block[2] = self.fmm_attributes.global_dimensions[2] as f32 - 0.5;
                    self.fmm_attributes.current_block[2] = self.fmm_attributes.global_dimensions[2]-1;
                }

                queue.write_buffer(
                    &self.buffers.get("fmm_attributes").unwrap(),
                    0,
                    bytemuck::cast_slice(&[self.fmm_attributes])
                );

                // Update hash tables.
                let (offset_hash_table, vec_to_offset_table, ivec_offset_hash_table) =
                    create_hash_table(4, 4, 4, self.fmm_attributes.global_dimensions[0], self.fmm_attributes.global_dimensions[1], self.fmm_attributes.global_dimensions[2] );

                queue.write_buffer(
                    &self.buffers.get("index_hash_table").unwrap(),
                    0,
                    bytemuck::cast_slice(&ivec_offset_hash_table)
                );
                // queue.write_buffer(
                //     &self.buffers.get("vec_to_offset").unwrap(),
                //     0,
                //     bytemuck::cast_slice(&vec_to_offset_table)
                // );
        }

        // Block location.
          
        let g_pressed = input.key_state(&Key::G);
        let h_pressed = input.key_state(&Key::H);
        let j_pressed = input.key_state(&Key::J);
        let y_pressed = input.key_state(&Key::Y);
        let u_pressed = input.key_state(&Key::U);
        let m_pressed = input.key_state(&Key::M);

        let mut block_x_pos: f32 = self.current_block[0];
        let mut block_y_pos: f32 = self.current_block[1];
        let mut block_z_pos: f32 = self.current_block[2];

        if !g_pressed.is_none() {block_x_pos = block_x_pos-time_offset; }
        if !h_pressed.is_none() {block_z_pos = block_z_pos+time_offset; }
        if !j_pressed.is_none() {block_x_pos = block_x_pos+time_offset; }
        if !y_pressed.is_none() {block_z_pos = block_z_pos-time_offset; }
        if !u_pressed.is_none() {block_y_pos = block_y_pos+time_offset; }
        if !m_pressed.is_none() {block_y_pos = block_y_pos-time_offset; }

        let mut block_pos: [f32;3] = [block_x_pos, block_y_pos, block_z_pos] ;

        if (block_x_pos >= 0.0 && block_x_pos < self.fmm_attributes.global_dimensions[0] as f32) &&
           (block_y_pos >= 0.0 && block_y_pos < self.fmm_attributes.global_dimensions[1] as f32) &&
           (block_z_pos >= 0.0 && block_z_pos < self.fmm_attributes.global_dimensions[2] as f32) {
                self.current_block = block_pos; 
                self.fmm_attributes.current_block  = [block_x_pos as u32, block_y_pos as u32, block_z_pos as u32];
                queue.write_buffer(
                    &self.buffers.get("fmm_attributes").unwrap(),
                    0,
                    bytemuck::cast_slice(&[self.fmm_attributes])
                );
        }

        // let enter_pressed = input.key_state(&Key::Return);
        // if !enter_pressed.is_none() {self.update_data_generator = (self.update_data_generator + 1) % 10; }

        //++ let add_pressed = input.key_state(&Key::Comma);
        //++ let minus_pressed = input.key_state(&Key::Period);
        //++ if !add_pressed.is_none() {
        //++     let value: f32 = self.triangle_index+time_offset;
        //++     if value < self.triangle_data.len() as f32 { 
        //++         self.triangle_index = value;
        //++         println!("{:?}", self.triangle_index);
        //++         queue.write_buffer(
        //++             &self.buffers.get("wood").unwrap(),
        //++             0,
        //++             bytemuck::cast_slice(&[self.triangle_data[self.triangle_index as usize]])
        //++         );
        //++     }
        //++ }
        //++ if !minus_pressed.is_none() {
        //++     let value: f32 = self.triangle_index-time_offset;
        //++     if value > 0.0 { 
        //++         self.triangle_index = value;
        //++         println!("{:?}", self.triangle_index);
        //++         queue.write_buffer(
        //++             &self.buffers.get("wood").unwrap(),
        //++             0,
        //++             bytemuck::cast_slice(&[self.triangle_data[self.triangle_index as usize]])
        //++         );
        //++     }
        //++ }

        let mut increase_fmm_step = false;

        let p_pressed = input.key_state(&Key::P);
        if !p_pressed.is_none() {
            self.changed = (self.changed + 1) % 10;
            if self.changed > 5 { increase_fmm_step = true; self.changed = 0; }
        }

        //if increase_fmm_step {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("FMM update encoder.") });

        if !self.data_loaded {
            if let Some(ref query_sets) = self.query_sets {
                encoder.write_timestamp(&query_sets.timestamp, 0);
            }

            self.fmm_data_generator.dispatch(&self.fmm_data_generator_bind_groups,
                        &mut encoder,
                        1,
                        1,
                        1
            ); 

            if let Some(ref query_sets) = self.query_sets {
                encoder.write_timestamp(&query_sets.timestamp, 1);
            }

            if let Some(ref query_sets) = self.query_sets {
                encoder.write_timestamp(&query_sets.timestamp, 2);
            }

            self.fmm_debug_pipeline.dispatch(&self.fmm_debug_bind_groups,
                        &mut encoder,
                        1,
                        1,
                        1
            ); 

            if let Some(ref query_sets) = self.query_sets {
                encoder.write_timestamp(&query_sets.timestamp, 3);
            }
            
            if let Some(ref query_sets) = self.query_sets {
                let timestamp_query_count = 4;
                encoder.resolve_query_set(
                        &query_sets.timestamp,
                        0..timestamp_query_count,
                        &query_sets.query_buffer,
                        0,
                        );
            }

            queue.submit(Some(encoder.finish()));

            //++let fmm_attributes = FMM_Attributes{ global_dimensions: BLOCK_DIMENSIONS,
            //++                                     offset_hash_table_size: 160 as u32,
            //++                                     current_block: [1,0,0],
            //++                                     vec_to_offset_table_size: 216 as u32,
            //++};

            //++queue.write_buffer(
            //++    &self.buffers.get("fmm_attributes").unwrap(),
            //++    0,
            //++    bytemuck::cast_slice(&[fmm_attributes])
            //++);

            if let Some(ref query_sets) = self.query_sets {
                // We can ignore the future as we're about to wait for the device.
                //
                let result = to_vec::<QueryData>(&device,
                        &queue,
                        &query_sets.query_buffer,
                        0 as wgpu::BufferAddress,
                        (std::mem::size_of::<QueryData>() as usize) as wgpu::BufferAddress);
                //println!("result :: {:?}", result);
                for (i, elem) in result[0].timestamps.iter().enumerate() {
                    let nanoseconds =
                        (elem.end - elem.start) as f32 * query_sets.timestamp_period;
                    let microseconds = nanoseconds / 1000.0;
                    let milli = microseconds / 1000.0;
                    //println!("{:?} time is {:?} micro seconds.", i, microseconds);
                    println!("{:?} time is {:?} milli seconds.", i, milli);
                }
            }

            // Get the counter values.
            let histogram = self.histogram.get_values(device, queue);
            self.debug_point_count = histogram[0];
            self.debug_triangle_draw_count = histogram[1];
            self.data_loaded = true;
        }
        //if increase_fmm_step {
    }
//                        encoder.copy_buffer_to_texture(
//                        wgpu::BufferCopyView {
//                            buffer: &self.buffers.get(BUFFERS.ray_march_output_buffer.name).unwrap().buffer,
//                            layout: wgpu::TextureDataLayout {
//                                offset: 0,
//                                bytes_per_row: CAMERA_RESOLUTION.0 * 4,
//                                rows_per_image: CAMERA_RESOLUTION.1,
//                            },
//                        },
//                        wgpu::TextureCopyView{
//                            texture: &self.textures.get(TEXTURES.ray_texture.name).unwrap().texture,
//                            mip_level: 0,
//                            origin: wgpu::Origin3d::ZERO,
//                        },
//                        wgpu::Extent3d {
//                            width: CAMERA_RESOLUTION.0,
//                            height: CAMERA_RESOLUTION.1,
//                            depth: 1,
//                    });
}

fn create_buffers(device: &wgpu::Device,
                  block_dimension: [u32 ; 3],
                  local_dimension: [u32 ; 3],
                  debug_point_output_size: u32,
                  buffers: &mut HashMap<String, wgpu::Buffer>) {

        // Define the bank conflict free prefix sum temp array size.
        //let block_dimension_size_bkf = block_dimension[0] * block_dimension[1] * block_dimension[2] + 
        //                               ((block_dimension[0] * block_dimension[1] * block_dimension[2]) >> 4);
        //println!("block_dimension_size_bkf == {}", block_dimension_size_bkf);
        let block_dimension_size = block_dimension[0] * block_dimension[1] * block_dimension[2];
        println!("block_dimension_size == {}", block_dimension_size);

        // layout(set = 0, binding = 1) buffer Prefix_sums  {
        // This buffer holds the temporary bank confict free data.
        buffers.insert(
            "prefix_sum_temp".to_string(),
            buffer_from_data::<u32>(
            &device,
            &vec![0 as u32; block_dimension_size as usize],
            //&vec![0; block_dimension_size_bkf * 4 as usize],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        // layout(set = 0, binding = 3) buffer Points_out {
        buffers.insert(
            "debug_points_output".to_string(),
            buffer_from_data::<OutputVertex>(
            &device,
            &vec![OutputVertex { pos: [0.0, 0.0, 0.0], color_point_size: 0 } ; (1024000*2) as usize],
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            None)
        );

        let number_of_blocks = block_dimension[0] * block_dimension[1] * block_dimension[2]; 
        let number_of_nodes = number_of_blocks * local_dimension[0] * local_dimension[1] * local_dimension[2]; 

        println!("Number of FMM_Nodes == {}", number_of_nodes);

        // layout(set = 0, binding = 4) buffer FMM_Nodes {
        buffers.insert(
            "fmm_nodes".to_string(),
            buffer_from_data::<FMM_Node>(
            &device,
            &vec![FMM_Node {value: 1000000.0, tag: FAR} ; number_of_nodes as usize],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        println!("CREATING BLOCKS");
        println!("number of blocks == {:?}", number_of_blocks);

        let mut test_blocks: Vec<FMM_Block> = vec![FMM_Block{ index: 0, band_points_count: 0} ; number_of_blocks as usize * 2];

        let mut index_counter: u32 = 0;
        for i in 0..number_of_blocks as usize {
            test_blocks[i] = FMM_Block{ index: i as u32, band_points_count: 0 as u32};
        };

        buffers.insert(
            "fmm_blocks".to_string(),
            buffer_from_data::<FMM_Block>(
            &device,
            &test_blocks,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            None)
        );

        buffers.insert(
            "fmm_data_gen_params".to_string(),
            buffer_from_data::<[u32; 4]>(
            &device,
            &[[2036, 0, 0, 0]],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            None)
        );
}

// fn create_fmm_buffer(device: &wgpu::Device,
//                      name: String, 
//                      dimension: [u32; 3],
//                      element_size: u32,
//                      buffers: &mut HashMap<String, wgpu::Buffer>) {
// 
//         assert!(dimension[0] % 4 == 0 && dimension[1] % 4 == 0 && dimension[2] % 4 == 0, "Each dimension should be a multiple of 4.");  
// 
//         buffers.insert(
//             name,
//             buffer_from_data::<f32>(
//             &device,
//             &vec![0 as f32 ; dimension[0] as usize * dimension[1] as usize * dimension[2] as usize],
//             wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
//             None)
//         );
// }

pub struct SphereTracerPipeline {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::ComputePipeline,
}

impl SphereTracerPipeline {

    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_entries(&self) -> &Vec<Vec<wgpu::BindGroupLayoutEntry>> {
        &self.layout_entries
    }

    pub fn dispatch(&self, bind_groups: &Vec<wgpu::BindGroup>,
                    encoder: &mut wgpu::CommandEncoder,
                    x: u32, y: u32, z: u32) {

        let mut pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor { label: None}
        );
        pass.set_pipeline(&self.pipeline);
        for (e, bgs) in bind_groups.iter().enumerate() {
            pass.set_bind_group(e as u32, &bgs, &[]);
        }
        pass.dispatch(x, y, z)
    }

    pub fn init(device: &wgpu::Device) -> Self {

        let mut comp_module = wgpu::include_spirv_raw!("../../shaders/spirv/sphere_tracer_fmm.comp.spv");

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                vec![
                    // layout(set = 0, binding = 0) uniform RayCamera {
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            },
                        count: None,
                    },
                    // layout(set = 0, binding = 1) buffer FMM_Nodes {
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None, // wgpu::BufferSize::new(temp_prefix_sum_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 2) buffer OffsetTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 3) buffer VecToHashTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set=0, binding=4) uniform FMM_Attributes
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(12),
                            },
                        count: None,
                    },
                    //wgpu::BindGroupLayoutEntry {
                    //    binding: 2,
                    //    visibility: wgpu::ShaderStages::COMPUTE,
                    //    ty: wgpu::BindingType::StorageTexture {
                    //        access: wgpu::StorageTextureAccess::ReadOnly,
                    //        format: wgpu::TextureFormat::Rgba32Float,
                    //        view_dimension: wgpu::TextureViewDimension::D2,
                    //    },
                    //    count: None,
                    //},
                    //layout(set = 0, binding = 5) buffer RayOutputBuffer {
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(3193724),
                        },
                        count: None,
                    },
                ],
        ];
        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        // Create the pipeline.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("sphere_tracer_fmm_pipeline"),
            layout: Some(&pipeline_layout),
            module: unsafe { &device.create_shader_module_spirv(&comp_module) },
            entry_point: "main",
        });


        Self {
            layout_entries, 
            bind_group_layouts, 
            pipeline,
        }
    }
}

/// Struct for fmm development version.
pub struct FMM_debug_pipeline {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::ComputePipeline,
}

impl FMM_debug_pipeline {

    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_entries(&self) -> &Vec<Vec<wgpu::BindGroupLayoutEntry>> {
        &self.layout_entries
    }

    pub fn dispatch(&self, bind_groups: &Vec<wgpu::BindGroup>,
                    encoder: &mut wgpu::CommandEncoder,
                    x: u32, y: u32, z: u32) {

        let mut pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor { label: None}
        );
        pass.set_pipeline(&self.pipeline);
        for (e, bgs) in bind_groups.iter().enumerate() {
            pass.set_bind_group(e as u32, &bgs, &[]);
        }
        pass.dispatch(x, y, z)
    }

    pub fn init(device: &wgpu::Device) -> Self {

        let mut comp_module = wgpu::include_spirv_raw!("../../shaders/spirv/fmm.comp.spv");

        // GLSL, validation disabled.
        //comp_module.flags = wgpu::ShaderFlags::empty();

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // layout(set = 0, binding = 0) uniform Dimensions. 
                vec![
                    // layout(set=0, binding=0) uniform camerauniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(16),
                            },
                        count: None,
                    },
                    // layout(set = 0, binding = 1) buffer Prefix_sums
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, // wgpu::BufferSize::new(temp_prefix_sum_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 2) buffer Points_out
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(3193724),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 3) buffer FMM_Nodes
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_nodes_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 4) buffer FMM_Blocks
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 5) buffer Counters
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 6) buffer OffsetTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 7) buffer VecToHashTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set=0, binding=8) uniform FMM_Attributes
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(12),
                            },
                        count: None,
                    },
                ],
        ];
        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        // Create the pipeline.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("sphere_tracer_fmm_pipeline"),
            layout: Some(&pipeline_layout),
            module: unsafe { &device.create_shader_module_spirv(&comp_module) },
            entry_point: "main",
        });

        Self {
            layout_entries, 
            bind_group_layouts, 
            pipeline,
        }
    }
}

/// Struct for fmm_data_generato development version.
pub struct FMM_data_generator_debug_pipeline {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::ComputePipeline,
}

impl FMM_data_generator_debug_pipeline {

    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_entries(&self) -> &Vec<Vec<wgpu::BindGroupLayoutEntry>> {
        &self.layout_entries
    }

    pub fn dispatch(&self, bind_groups: &Vec<wgpu::BindGroup>,
                    encoder: &mut wgpu::CommandEncoder,
                    x: u32, y: u32, z: u32) {

        let mut pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor { label: None}
        );
        pass.set_pipeline(&self.pipeline);
        for (e, bgs) in bind_groups.iter().enumerate() {
            pass.set_bind_group(e as u32, &bgs, &[]);
        }
        pass.dispatch(x, y, z)
    }

    pub fn init(device: &wgpu::Device) -> Self {

        let mut comp_module = wgpu::include_spirv_raw!("../../shaders/spirv/fmm_data_generator.comp.spv");

        // GLSL, validation disabled.
        //comp_module.flags = wgpu::ShaderFlags::empty();

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // layout(set=0, binding=0) uniform camerauniform {
                vec![
                    // layout(set=0, binding=0) uniform camerauniform {
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(16),
                            },
                        count: None,
                    },
                    //layout(set = 0, binding = 1) buffer Counters {
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 2) buffer Points_out {
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    //layout(set = 0, binding = 3) buffer FMM_Nodes
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    //++wgpu::BindGroupLayoutEntry {
                    //++    binding: 4,
                    //++    visibility: wgpu::ShaderStages::COMPUTE,
                    //++    ty: wgpu::BindingType::Buffer {
                    //++        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    //++        has_dynamic_offset: false,
                    //++        min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
                    //++    },
                    //++    count: None,
                    //++},
                    //layout(set = 0, binding = 4) buffer Triangle_data
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
                        },
                        count: None,
                    },
                    //layout(set = 0, binding = 5) readonly buffer OffsetTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
                        },
                        count: None,
                    },
                    //layout(set = 0, binding = 6) readonly buffer VecToHashTable
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 5) uniform texture1D z1z2_texture;
                    //++wgpu::BindGroupLayoutEntry {
                    //++    binding: 5,
                    //++    visibility: wgpu::ShaderStages::COMPUTE,
                    //++    ty: wgpu::BindingType::StorageTexture {
                    //++        access: wgpu::StorageTextureAccess::ReadOnly,
                    //++        format: wgpu::TextureFormat::Rgba32Float,
                    //++        view_dimension: wgpu::TextureViewDimension::D1,
                    //++    },
                    //++    count: None,
                    //++},
                    // layout(set=0, binding=8) uniform FMM_Attributes {
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(24),
                            },
                        count: None,
                    },
                    //layout(set=0, binding=8) uniform General_params {
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //min_binding_size: wgpu::BufferSize::new(24),
                            },
                        count: None,
                    },
                    // layout(set = 0, binding = 6) uniform sampler z1z2_texture_sampler;
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 6,
                    //     visibility: wgpu::ShaderStages::COMPUTE,
                    //     ty: wgpu::BindingType::Sampler {
                    //         filtering: true,
                    //         comparison: false,
                    //     },
                    //     count: None,
                    // },
                ],
        ];
        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        // Create the pipeline.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("fmm_data_generator_debug_pipeline"),
            layout: Some(&pipeline_layout),
            module: unsafe { &device.create_shader_module_spirv(&comp_module) },
            entry_point: "main",
        });

        Self {
            layout_entries, 
            bind_group_layouts, 
            pipeline,
        }
    }
}

fn main() {
    ws::run_loop::<FMM_App, BasicLoop, FMM_Features>(); 
}
