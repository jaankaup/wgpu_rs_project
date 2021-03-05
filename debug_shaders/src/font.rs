use jaankaup_core::render_pipelines::create_bind_group_layouts;

pub struct Font_pipeline {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::ComputePipeline,
}

impl Font_pipeline {

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
                //comp_module: &wgpu::ShaderModule,
                ) -> Self {

        let comp_module = wgpu::include_spirv!("../../shaders/spirv/font.comp.spv");

        // Define all bind grout entries for pipeline and bind groups.
        let layout_entries = vec![
                // Set 0
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
                    // Counter 
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
                    // Output: ? 
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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
            label: Some("font_shader_pipeline"),
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
