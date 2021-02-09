use crate::render_pipelines::*;

struct Custom3DTexture {
    pub layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pub pipeline: wgpu::ComputePipeline,
}

impl Custom3DTexture {
    pub fn init(device: &wgpu::Device, comp_module: &wgpu::ShaderModule) -> Self {

        // Define bind grout entries. Output buffer.
        let layout_entries = vec![
                    vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
        ];

        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        // Create the pipeline.
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &comp_module,
            entry_point: "main",
        });

        Self {
            layout_entries: layout_entries, 
            bind_group_layouts: bind_group_layouts, 
            pipeline: pipeline,

        }
    }
}
