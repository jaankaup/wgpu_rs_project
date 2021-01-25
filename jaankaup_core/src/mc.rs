/// A struct for marching cubes algorithm purposes.
pub struct MarchingCubes {
    pipeline: wgpu::ComputePipeline,
    density_buffer: Option<wgpu::Buffer>,
}

impl MarchingCubes {

    pub fn init(device: &wgpu::Device, mc_shader: &wgpu::ShaderModule) -> Self {
        Self {
            pipeline: MarchingCubes::create_pipeline(&device, &mc_shader),
            density_buffer: None,
        }
    }

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
