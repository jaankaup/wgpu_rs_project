// TODO: define shader_modules
use crate::buffer::*;
use crate::texture as jaankaup;
use crate::misc::create_vb_descriptor;

/// Resources for rendering a single texture on the whole screen.
pub struct TwoTriangles {
    pipeline: wgpu::RenderPipeline,
    draw_buffer: wgpu::Buffer,
}

impl TwoTriangles {

    /// Creates resources.
    pub fn init(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        
        Self {
            pipeline: TwoTriangles::create_pipeline(&device, &sc_desc),
            draw_buffer: TwoTriangles::create_screen_texture_buffer(&device),
        }
    }

    /// Create a bind group for TwoTriangles using 2d texture.
    pub fn create_bind_group(device: &wgpu::Device, texture: &jaankaup::Texture) -> wgpu::BindGroup {

        let bind_group_layout = TwoTriangles::get_bind_group_layout(&device);

        // Create bindings.
        let bind_group_entry0 = wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
        };

        let bind_group_entry1 = wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[bind_group_entry0, bind_group_entry1],
            label: None,
        });

        bind_group
    }

    /// Creates the BindGroupLayout for TwoTriangles pipeline and bindgroup.
    fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {

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
                label: Some("two_triangles_bind_group_layout"),
        });

        bind_group_layout

    }

    pub fn draw(&self,
                encoder: &mut wgpu::CommandEncoder,
                frame: &wgpu::SwapChainTexture,
                depth_texture: &jaankaup::Texture,
                bind_group: &wgpu::BindGroup,
                clear: bool) {

        let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("two_triangles_rendes_pass_descriptor"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: match clear {
                                        true => {
                                            wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 0.0,
                                                g: 0.0,
                                                b: 0.0,
                                                a: 1.0,
                                            })
                                        }
                                        false => {
                                            wgpu::LoadOp::Load
                                        }
                                    },
                                    store: true,
                                },
                        }
                    ],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                            load: match clear { true => wgpu::LoadOp::Clear(1.0), false => wgpu::LoadOp::Load },
                            store: true,
                    }),
                    stencil_ops: None,
                    }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            // Set vertex buffer.
            render_pass.set_vertex_buffer(
                0,
                self.draw_buffer.slice(..)
            );

            render_pass.draw(0..6, 0..1);
    }

    /// Load and compile shaders for TwoTriangles.
    fn load_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    
        let vertex_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.vert.spv");
        let fragment_shader_src = wgpu::include_spirv!("../../shaders/spirv/screen_texture.frag.spv");
    
        let vert = device.create_shader_module(&vertex_shader_src);
        let frag = device.create_shader_module(&fragment_shader_src);
        
        (vert, frag)
    }
    
    /// Create the pipeline for TwoTriangles.
    fn create_pipeline(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
    
        let (vs_module, fs_module) = TwoTriangles::load_shaders(&device);
        
        // Create bind group layout for pipeline.
        let bind_group_layout = TwoTriangles::get_bind_group_layout(&device);
        
        // Create pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("two_triangles_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create stride and vertex attribute descriptors.
        let (stride, vb_desc) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float4, wgpu::VertexFormat::Float4]
        );
        
        // Create the pipeline.
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("two_triangles_pipeline"),
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
                index_format: None, //Some(wgpu::IndexFormat::Uint16),
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: stride,
                        step_mode: wgpu::InputStepMode::Vertex, 
                        attributes: &vb_desc,
                    }],
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
