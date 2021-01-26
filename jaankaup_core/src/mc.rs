use bytemuck::{Zeroable, Pod};
use crate::buffer::buffer_from_data;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct McUniform {
    pub base_position: cgmath::Vector4<f32>,
    pub isovalue: f32,
    pub cube_length: f32,
    pub future_usage1: f32,
    pub future_usage2: f32,
}

unsafe impl Pod for McUniform {}
unsafe impl Zeroable for McUniform {}

/// Uniform data for marching cubes (set=0, binding=0).
pub struct McParams {
    params: McUniform,
    buffer: wgpu::Buffer,
}

impl McParams {

    /// Create an instance of McParams.
    pub fn init(device: &wgpu::Device, base_position: &cgmath::Vector4<f32>, isovalue: f32, cube_length: f32) -> Self {
        assert!(cube_length > 0.0, format!("cube_length ==  {} > 0.0", cube_length));

        let uniform = McUniform {
                base_position: *base_position,
                isovalue: isovalue,
                cube_length: cube_length,
                future_usage1: 0.0,
                future_usage2: 0.0,
        }; 

        Self {
            params: uniform,
            buffer: buffer_from_data::<McUniform>(
                &device,
                &[uniform],
                wgpu::BufferUsage::COPY_DST |wgpu::BufferUsage::UNIFORM,
                None),
        }
    }

    pub fn get_params(&self) -> &McUniform {
        &self.params
    }

    /// Updates the given mc-parameters and updates the buffer.
    pub fn update_params(
        &mut self,
        queue: &wgpu::Queue,
        base_position: &Option<cgmath::Vector4<f32>>,
        isovalue: &Option<f32>,
        cube_length: &Option<f32>) {

        // Update params.
        if let Some(position) = *base_position {
            self.params.base_position = position;
        }
        if let Some(iso) = *isovalue {
            self.params.isovalue = iso;
        }
        if let Some(length) = *cube_length {
            assert!(length > 0.0, format!("length ==  {} > 0.0", length));
            self.params.cube_length = length;
        }

        // Update the buffer.
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.params])
        );
    }
}

/// A struct for marching cubes algorithm purposes.
pub struct MarchingCubes {
    pipeline: wgpu::ComputePipeline,
    counter_buffer: Option<wgpu::Buffer>,
    density_buffer: Option<wgpu::Buffer>,
}

impl MarchingCubes {

    pub fn init(device: &wgpu::Device, mc_shader: &wgpu::ShaderModule) -> Self {
        Self {
            pipeline: MarchingCubes::create_pipeline(&device, &mc_shader),
            counter_buffer: None,
            density_buffer: None,
        }
    }

//    pub fn create_bind_groups(
//        device: &device,

    /// Creates the BindGroupLayout for marching cubes pipeline and bindgroup.
    fn get_bind_group_layout(device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout> {

        let mut layouts = Vec::<wgpu::BindGroupLayout>::new();

        layouts.push(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Mc_test bind group layout 0"),
            })
        );
        layouts.push(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Mc_test bind group layout 1"),
            })
        );
        layouts
    }

    /// Create the pipeline for marching cubes.
    fn create_pipeline(
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule)
    -> wgpu::ComputePipeline {
    
        let bind_group_layouts = MarchingCubes::get_bind_group_layout(&device);
        
        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mc cubes test layout"),
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(), // &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create the pipeline.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("mc_pipeline"),
            layout: Some(&pipeline_layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader_module,
                entry_point: "main",
            },
        });

        pipeline
    }
}
