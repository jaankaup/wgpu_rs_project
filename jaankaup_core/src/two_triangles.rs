// TODO: define shader_modules

use crate::buffer::*;
use crate::misc::create_vb_descriptor;

pub struct TwoTriangles {
    pipeline: wgpu::RenderPipeline,
    draw_buffer: wgpu::Buffer,
}

impl TwoTriangles {

    pub fn init(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        
        Self {
            pipeline: TwoTriangles::create_pipeline(&device, &sc_desc),
            draw_buffer: TwoTriangles::create_screen_texture_buffer(&device),
        }
    }

    /// Load shaders and compile.
    fn load_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    
        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.frag.spv");
    
        let vert = device.create_shader_module(&vertex_shader_src);
        let frag = device.create_shader_module(&fragment_shader_src);
        
        (vert, frag)
    }
    
    /// Creates pipeline for drawing texture on the screen (two triangles).
    fn create_pipeline(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
    
        let (vs_module, fs_module) = TwoTriangles::load_shaders(&device);
        
        // Create bind group layout for pipeline.
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
                label: None,
        });
        
        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create stride and vertex attribute descriptors.
        let (stride, vb_desc) =  create_vb_descriptor(&vec![wgpu::VertexFormat::Float3]);
        
        // Create the pipeline.
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[sc_desc.format.into()],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: Some(wgpu::IndexFormat::Uint16),
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: stride,
                        step_mode: wgpu::InputStepMode::Vertex, 
                        attributes: &vb_desc,
                    }], // TODO: create function!
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        pipeline
    }
    
    /// Creates a buffer for screen filling texture (two triangles).
    pub fn create_screen_texture_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    
        buffer_from_data::<f32>(
            device,
            // gl_Position     |    point_pos
            &[-1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
               1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
               1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
               1.0,  1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
              -1.0,  1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
              -1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
            ],
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC,
            None
        )
    }
}
