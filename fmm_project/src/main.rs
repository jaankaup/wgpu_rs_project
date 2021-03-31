use jaankaup_core::wgpu;
use render_shaders::Render_vvvc;
use jaankaup_core::buffer::{buffer_from_data, to_vec};
use std::collections::HashMap;
use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};

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
use model_loader::load_triangles_from_obj;
use bytemuck::{Pod, Zeroable};

//static DEBUG_BUFFER_SIZE: u32 = 33554432;
//TODO: rename. This is not the buffer size. It's the number on poinst, and the number of line
//triangle points.
static DEBUG_BUFFER_SIZE: u32   = 1024000; //4194300; // 1048575; //33554416;
static DEBUG_BUFFER_OFFSET: u32 = 1024000; // 2097151 / 2 ~= 1048574
static BLOCK_DIMENSIONS: [u32; 3] = [8, 8, 8];
//8388607

// Redefine needed features for this application.
struct FMM_Features {}
impl WGPUFeatures for FMM_Features {
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

// The fmm application.
struct FMM_App {
    depth_texture: JTexture,
    camera: Camera,
    buffers: HashMap<String, wgpu::Buffer>,
    render_vvvc_point_pipeline: Render_vvvc, 
    render_vvvc_point_bind_groups: Vec<wgpu::BindGroup>,
    render_vvvc_triangle_pipeline: Render_vvvc, 
    render_vvvc_triangle_bind_groups: Vec<wgpu::BindGroup>,
    fmm_debug_pipeline: FMM_debug_pipeline,
    fmm_debug_bind_groups: Vec<wgpu::BindGroup>,
//    histogram: Histogram, 
    debug_point_count: u32,
    debug_triangle_draw_count: u32,
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
        camera.set_movement_sensitivity(0.002);

        // Create buffers for fmm alogrithm.
        create_buffers(&configuration.device,
                       BLOCK_DIMENSIONS, // block_dimension: [u32 ; 3],
                       [4, 4, 4], //local_dimension: [u32 ; 3],
                       DEBUG_BUFFER_SIZE, //debug_point_output_size: u32,
                       &mut buffers
        );

        // Create render pipelines for debugger.
        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvc_camera.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/render_vvvc_camera.frag.spv");

        // The point pipeline.
        let render_vvvc_point_pipeline = Render_vvvc::init(
                    &configuration.device,
                    &configuration.sc_desc,
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src),
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
                    &configuration.device.create_shader_module(&vertex_shader_src),
                    &configuration.device.create_shader_module(&fragment_shader_src),
                    wgpu::PrimitiveTopology::TriangleList
        );
        let render_vvvc_triangle_bind_groups = render_vvvc_triangle_pipeline.create_bind_groups(
            &configuration.device,
            &camera.get_camera_uniform(&configuration.device)
        );

        let mut debug_point_count = 2;
        let mut debug_triangle_draw_count = DEBUG_BUFFER_SIZE;

        // Create histogram for fmm debug.
        //let mut histogram = Histogram::init(&configuration.device, &vec![0, DEBUG_BUFFER_OFFSET]); 

        // let block_dimension_size = (BLOCK_DIMENSIONS[0] * BLOCK_DIMENSIONS[1] * BLOCK_DIMENSIONS[2] + 
        //                            ((BLOCK_DIMENSIONS[0] * BLOCK_DIMENSIONS[1] * BLOCK_DIMENSIONS[2]) >> 4)) 
        //                            * std::mem::size_of::<u32>() as u32;
        // let output_vertex_size = std::mem::size_of::<OutputVertex>() as u32 * DEBUG_BUFFER_SIZE * 2;
        // let fmm_block_size = std::mem::size_of::<FMM_Block>() as u32 * BLOCK_DIMENSIONS[0] * BLOCK_DIMENSIONS[1] * BLOCK_DIMENSIONS[2];
        // let fmm_nodes_size = std::mem::size_of::<FMM_Node>() as u32 * BLOCK_DIMENSIONS[0] * BLOCK_DIMENSIONS[1] * BLOCK_DIMENSIONS[2] * 64;

        let fmm_debug_pipeline = FMM_debug_pipeline::init(&configuration.device);

        let fmm_debug_bind_groups =
                create_bind_groups(
                    &configuration.device, 
                    &fmm_debug_pipeline.get_bind_group_layout_entries(),
                    &vec![
                        vec![
                            //&buffers.get("").as_entire_binding(),
                            &camera.get_camera_uniform(&configuration.device).as_entire_binding(),
                            &buffers.get("prefix_sum_temp").unwrap().as_entire_binding(),
                            &buffers.get("debug_points_output").unwrap().as_entire_binding(),
                            &buffers.get("fmm_nodes").unwrap().as_entire_binding(),
                            &buffers.get("fmm_blocks").unwrap().as_entire_binding(),
                        ], 
                    ]
        );

        Self {
            depth_texture,
            camera,
            buffers,
            render_vvvc_point_pipeline, 
            render_vvvc_point_bind_groups,
            render_vvvc_triangle_pipeline, 
            render_vvvc_triangle_bind_groups,
            fmm_debug_pipeline,
            fmm_debug_bind_groups,
            //histogram,
            debug_point_count,
            debug_triangle_draw_count,
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

         draw(&mut encoder,
              &frame,
              &self.depth_texture,
              &self.render_vvvc_point_bind_groups,
              &self.render_vvvc_point_pipeline.get_pipeline(),
              &self.buffers.get("debug_points_output").unwrap(),
              2..self.debug_point_count,
              //3..3000,
              true
         );

         draw(&mut encoder,
              &frame,
              &self.depth_texture,
              &self.render_vvvc_triangle_bind_groups,
              &self.render_vvvc_triangle_pipeline.get_pipeline(),
              &self.buffers.get("debug_points_output").unwrap(),
              DEBUG_BUFFER_SIZE..self.debug_triangle_draw_count,
              false
         );
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

        //self.histogram.set_values_cpu_version(&queue, &vec![0, DEBUG_BUFFER_OFFSET]);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("FMM update encoder.") });
        
        self.fmm_debug_pipeline.dispatch(&self.fmm_debug_bind_groups,
                    &mut encoder,
                    1,
                    1,
                    1
        ); 

        queue.submit(Some(encoder.finish()));

        let histogram = to_vec::<OutputVertex>(&device,
                                   &queue,
                                   &self.buffers.get("debug_points_output").unwrap(),
                                   0 as wgpu::BufferAddress,
                                   (std::mem::size_of::<OutputVertex>() * 2) as wgpu::BufferAddress);


        self.debug_point_count = histogram[0].color_point_size;
        self.debug_triangle_draw_count = histogram[1].color_point_size;
    }
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
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );

        // layout(set = 0, binding = 3) buffer Points_out {
        buffers.insert(
            "debug_points_output".to_string(),
            buffer_from_data::<OutputVertex>(
            &device,
            &vec![OutputVertex { pos: [0.0, 0.0, 0.0], color_point_size: 0 } ; (1024000*2) as usize],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        let number_of_blocks = block_dimension[0] * block_dimension[1] * block_dimension[2]; 
        let number_of_nodes = number_of_blocks * local_dimension[0] * local_dimension[1] * local_dimension[2]; 

        // layout(set = 0, binding = 4) buffer FMM_Nodes {
        buffers.insert(
            "fmm_nodes".to_string(),
            buffer_from_data::<FMM_Node>(
            &device,
            &vec![FMM_Node {value: 0.0, tag: 0} ; number_of_nodes as usize],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );

        let active_block_ids: Vec<u32> = vec![0, 2, 124, 127, 128, 168, 300];

        println!("CREATING BLOCKS");

        let mut test_blocks: Vec<FMM_Block> = Vec::new();

        let mut index_counter: u32 = 0;
        for k in 0..block_dimension[0] {
        for j in 0..block_dimension[1] {
        for i in 0..block_dimension[2] {
            if index_counter < 8*8*8 {
                //if k == 1 || (j == 1 || i == 0) { 
                if active_block_ids.contains(&index_counter) {
                    test_blocks.push(FMM_Block{ index: index_counter, band_points_count: 3});  
                    println!("{}", index_counter);
                }
                else {
                    test_blocks.push(FMM_Block{ index: index_counter, band_points_count: 0});  
                }
            }
            index_counter = index_counter + 1;
        }}};

        // Populate the 'Active FMM_Blocks'.
        for i in 0..number_of_blocks {
            test_blocks.push(FMM_Block{ index: 777, band_points_count: 0});  
        }

        buffers.insert(
            "fmm_blocks".to_string(),
            buffer_from_data::<FMM_Block>(
            &device,
            &test_blocks,
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
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
//             wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
//             None)
//         );
// }

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

        let comp_module = wgpu::include_spirv!("../../shaders/spirv/fmm.comp.spv");

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // layout(set = 0, binding = 0) uniform Dimensions. 
                vec![
                    // layout(set=0, binding=0) uniform camerauniform {
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                            },
                        count: None,
                    },
                    // layout(set = 0, binding = 1) buffer Prefix_sums  {
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, // wgpu::BufferSize::new(temp_prefix_sum_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 2) buffer Points_out {
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(points_out_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 3) buffer FMM_Nodes {
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_nodes_size),
                        },
                        count: None,
                    },
                    // layout(set = 0, binding = 4) buffer FMM_Blocks {
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, //wgpu::BufferSize::new(fmm_blocks_size),
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
            label: Some("fmm_debug_pipeline"),
            layout: Some(&pipeline_layout),
            module: &device.create_shader_module(&comp_module),
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
