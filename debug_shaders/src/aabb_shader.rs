use jaankaup_core::render_pipelines::create_bind_group_layouts;

pub struct AABB_pipeline {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::ComputePipeline,
}

impl AABB_pipeline {

    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    pub fn init(device: &wgpu::Device,
                comp_module: &wgpu::ShaderModule,
                ) -> Self {

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // Set 0
                vec![wgpu::BindGroupLayoutEntry {
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
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
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
        ];
        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        // TODO: refactor?.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        // Create the pipeline.
        // TODO: refactor?.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("aabb_shader_pipeline"),
            layout: Some(&pipeline_layout),
            module: &comp_module,
            entry_point: "main",
        });

        Self {
            layout_entries, 
            bind_group_layouts, 
            pipeline,
        }
    }
}
