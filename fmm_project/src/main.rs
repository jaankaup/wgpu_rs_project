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
use model_loader::load_triangles_from_obj;
use bytemuck::{Pod, Zeroable};

//static DEBUG_BUFFER_SIZE: u32 = 33554432;
static DEBUG_BUFFER_SIZE: u32 = 33554428;
static DEBUG_BUFFER_OFFSET: u32 = 4194302;
//8388607

// Redefine needed features for this application.
struct FMM_Features {}
impl WGPUFeatures for FMM_Features {
}

#[derive(Copy, Clone)]
struct FMM_Block {
    //base_coord: [u32 ; 3],
    index: u32,
    band_points_count: u32,
}

unsafe impl bytemuck::Zeroable for FMM_Block {}
unsafe impl bytemuck::Pod for FMM_Block {}

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
    histogram: Histogram, 
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
        camera.set_movement_sensitivity(0.004);

        // Create buffers for fmm alogrithm.
        create_buffers(&configuration.device,
                       [4, 4, 4], // block_dimension: [u32 ; 3],
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

        let mut debug_point_count = 0;
        let mut debug_triangle_draw_count = DEBUG_BUFFER_OFFSET;

        // Create histogram for fmm debug.
        let mut histogram = Histogram::init(&configuration.device, &vec![0, DEBUG_BUFFER_OFFSET]); 

        let fmm_debug_pipeline = FMM_debug_pipeline::init(&configuration.device);

        let fmm_debug_bind_groups =
                create_bind_groups(
                    &configuration.device, 
                    &fmm_debug_pipeline.get_bind_group_layout_entries(),
                    &vec![
                        vec![
                            &buffers.get("dimensions_font").unwrap().as_entire_binding(),
                            &histogram.get_histogram().as_entire_binding(),
                            &camera.get_camera_uniform(&configuration.device).as_entire_binding(),
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
            histogram,
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
             0..self.debug_point_count,
             true
        );

        draw(&mut encoder,
             &frame,
             &self.depth_texture,
             &self.render_vvvc_triangle_bind_groups,
             &self.render_vvvc_triangle_pipeline.get_pipeline(),
             &self.buffers.get("debug_points_output").unwrap(),
             DEBUG_BUFFER_OFFSET..self.debug_triangle_draw_count,
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

        self.histogram.set_values_cpu_version(&queue, &vec![0, DEBUG_BUFFER_OFFSET]);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("FMM update encoder.") });
        
        self.fmm_debug_pipeline.dispatch(&self.fmm_debug_bind_groups,
                    &mut encoder,
                    1,
                    1,
                    1
        ); 

        queue.submit(Some(encoder.finish()));

        let histogram = to_vec::<u32>(&device,
                                   &queue,
                                   &self.histogram.get_histogram(),
                                   0 as wgpu::BufferAddress,
                                   8 as wgpu::BufferAddress);

        // println!("point_count == {}, triangle_point_count == {}", histogram[0], histogram[1]);

        self.debug_point_count = histogram[0];
        self.debug_triangle_draw_count = histogram[1];
    }
}

fn create_buffers(device: &wgpu::Device,
                  block_dimension: [u32 ; 3],
                  local_dimension: [u32 ; 3],
                  debug_point_output_size: u32,
                  buffers: &mut HashMap<String, wgpu::Buffer>) {

        // layout(set = 0, binding = 0) uniform Dimensions
        buffers.insert(
            "dimensions_font".to_string(),
            buffer_from_data::<u32>(
            &device,
            &block_dimension,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
            None)
        );

        // layout(set = 0, binding = 3) buffer Points_out {
        buffers.insert(
            "debug_points_output".to_string(),
            buffer_from_data::<f32>(
            &device,
            &vec![0 as f32 ; debug_point_output_size as usize],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            None)
        );

        let number_of_blocks = block_dimension[0] * block_dimension[1] * block_dimension[2]; 
        let number_of_nodes = number_of_blocks * local_dimension[0] * local_dimension[1] * local_dimension[2]; 

        // for i in 0..512 {
        //     println!("{} >> 16 + {} >> 8 == {}", i, i, (i >> 16) + (i >> 8));
        // }

        //let offset: u32 = 1;
        //for i in 0..64 {
        //    let mut ai: u32 = offset*(2*i+1)-1;
        //    let mut bi: u32 = offset*(2*i+2)-1;
        //    ai += ai / 16;
        //    bi += bi / 16;
        //    println!("offset == {}. thid == {}. ai == {}. bi == {}", offset, i, ai, bi);
        //}
        //
        
        // for i in 0..64 {
        //     let mut ai: u32 = i;
        //     let mut bi: u32 = i + 64;
        //     let mut bankOffsetA: u32 = (ai + (ai >> 4));
        //     let mut bankOffsetB: u32 = (bi + (bi >> 4));
        //     println!("{} :: bankOffsetA == {}", ai, bankOffsetA);
        //     println!("{} :: bankOffsetB == {}", bi, bankOffsetB);
        // }

        // layout(set = 0, binding = 4) buffer FMM_Nodes {
        buffers.insert(
            "fmm_nodes".to_string(),
            buffer_from_data::<FMM_Node>(
            &device,
            &vec![FMM_Node {value: 0.0, tag: 0} ; number_of_nodes as usize],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );

        let active_block_ids: Vec<u32> = vec![0, 2, 124, 127, 128, 168];;

        println!("CREATING BLOCKS");

        let mut test_blocks: Vec<FMM_Block> = Vec::new();

        let mut index_counter: u32 = 0;
        for k in 0..8 { //block_dimension[0] {
        for j in 0..8 { //block_dimension[1] {
        for i in 0..8 { //block_dimension[2] {
            if index_counter < 185 {
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

        buffers.insert(
            "fmm_blocks".to_string(),
            buffer_from_data::<FMM_Block>(
            &device,
            &test_blocks,
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );
}

fn create_fmm_buffer(device: &wgpu::Device,
                     name: String, 
                     dimension: [u32; 3],
                     element_size: u32,
                     buffers: &mut HashMap<String, wgpu::Buffer>) {

        assert!(dimension[0] % 4 == 0 && dimension[1] % 4 == 0 && dimension[2] % 4 == 0, "Each dimension should be a multiple of 4.");  

        buffers.insert(
            name,
            buffer_from_data::<f32>(
            &device,
            &vec![0 as f32 ; dimension[0] as usize * dimension[1] as usize * dimension[2] as usize],
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            None)
        );
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

    pub fn init(device: &wgpu::Device
                ) -> Self {

        let comp_module = wgpu::include_spirv!("../../shaders/spirv/fmm.comp.spv");

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // layout(set = 0, binding = 0) uniform Dimensions. 
                vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(3), //None,
                            },
                        count: None,
                    },
                    //layout(set = 0, binding = 1) buffer Counter
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(1), //None,
                        },
                        count: None,
                    },
                    //layout(set=0, binding=2) uniform camerauniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                            },
                        count: None,
                    },
                    //layout(set = 0, binding = 3) buffer Points_out.
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    //layout(set = 0, binding = 4) buffer FMM_Nodes {
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    //layout(set = 0, binding = 5) buffer FMM_Blocks {
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
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
