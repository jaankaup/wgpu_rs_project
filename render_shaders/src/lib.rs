use jaankaup_core::wgpu;
use jaankaup_core::render_pipelines::{
    draw,
    create_bind_groups,
    TestLayoutEntry,
    check_correspondence,
};
use jaankaup_core::misc::create_vb_descriptor;

/// A pipeline/layout entries for render shaders that only consume vvvvnnnn data.
pub struct Render_vvvvnnnn {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::RenderPipeline,
}

impl Render_vvvvnnnn {

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_entries(&self) -> &Vec<Vec<wgpu::BindGroupLayoutEntry>> {
        &self.layout_entries
    }

    pub fn create_bind_groups(&self, device: &wgpu::Device, camera_buffer: &wgpu::Buffer) -> Vec<wgpu::BindGroup> {
        create_bind_groups(
             &device, 
             &self.layout_entries,
             &vec![
                 vec![
                    &camera_buffer.as_entire_binding()
                 ]
             ]
        )
    }

    pub fn init(device: &wgpu::Device,
                sc_desc: &wgpu::SwapChainDescriptor,               
                wgsl_module: &wgpu::ShaderModule,
                //vs_module: &wgpu::ShaderModule,
                //fs_module: &wgpu::ShaderModule,
                ) -> Self {

        // Camera uniform.
        let layout_entries = vec![
                // Set 0
                vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            },
                        count: None,
                    },
                ],
        ];

        let (stride, attributes) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]
        );

        let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();

        for ve in layout_entries.iter() {
            bind_group_layouts.push(
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &ve,
                        label: None,
                    }
                )
            );
        }

        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wgsl_module,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: stride,
                        step_mode: wgpu::InputStepMode::Vertex, 
                        attributes: &attributes,
                    }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                clamp_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
                //clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &wgsl_module,
                entry_point: "main",
                targets: &[sc_desc.format.into()] // &[wgpu::ColorTargetState {
                //   format: sc_desc.format,
                //   alpha_blend: wgpu::BlendState::REPLACE,
                //   color_blend: wgpu::BlendState::REPLACE,
                //   write_mask: wgpu::ColorWrite::COLOR,
                //}],
                //targets: &[wgpu::ColorTargetState {
                //   format: sc_desc.format,
                //   alpha_blend: wgpu::BlendState::REPLACE,
                //   color_blend: wgpu::BlendState::REPLACE,
                //   write_mask: wgpu::ColorWrite::COLOR,
                //}],
            }),
        });

        Self {
            layout_entries: layout_entries,
            bind_group_layouts: bind_group_layouts,
            pipeline: pipeline,
        }
    }
}

/// A pipeline/layout entries for rendering point data vvvc (c :: color).
pub struct Render_vvvc {
    layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pipeline: wgpu::RenderPipeline,
}

impl Render_vvvc {

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn get_bind_group_layout_entries(&self) -> &Vec<Vec<wgpu::BindGroupLayoutEntry>> {
        &self.layout_entries
    }

    pub fn create_bind_groups(&self, device: &wgpu::Device, camera_buffer: &wgpu::Buffer) -> Vec<wgpu::BindGroup> {
        create_bind_groups(
             &device, 
             &self.layout_entries,
             &vec![
                 vec![
                    &camera_buffer.as_entire_binding()
                 ]
             ]
        )
    }

    pub fn init(device: &wgpu::Device,
                sc_desc: &wgpu::SwapChainDescriptor,               
                vs_module: &wgpu::ShaderModule,
                fs_module: &wgpu::ShaderModule,
                topology: wgpu::PrimitiveTopology,
                ) -> Self {

        // Camera uniform.
        let layout_entries = vec![
                // Set 0
                vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            },
                        count: None,
                    },
                ],
        ];

        let (stride, attributes) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float32x3, wgpu::VertexFormat::Uint32]
        );

        let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();

        for ve in layout_entries.iter() {
            bind_group_layouts.push(
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &ve,
                        label: None,
                    }
                )
            );
        }

        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vvvc point"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: stride,
                        step_mode: wgpu::InputStepMode::Vertex, 
                        attributes: &attributes,
                    }],
            },
            primitive: wgpu::PrimitiveState {
                topology: topology, //wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill, //Point,
                conservative: false,
                clamp_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
                //clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[sc_desc.format.into()] // &[wgpu::ColorTargetState {
                //   format: sc_desc.format,
                //   alpha_blend: wgpu::BlendState::REPLACE,
                //   color_blend: wgpu::BlendState::REPLACE,
                //   write_mask: wgpu::ColorWrite::COLOR,
                //}],
            }),
        });

        Self {
            layout_entries: layout_entries,
            bind_group_layouts: bind_group_layouts,
            pipeline: pipeline,
        }
    }
}
