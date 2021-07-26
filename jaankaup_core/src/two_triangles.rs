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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                        wgpu::RenderPassColorAttachment {
                                view: &frame.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: match clear {
                                        true => {
                                            wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 1.0,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
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

            render_pass.draw(0..3, 0..1);
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
        let (stride, attributes) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]
        );
        
        // Create the pipeline.
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("two_triangles_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: stride,
                        step_mode: wgpu::VertexStepMode::Vertex, 
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
                module: &fs_module,
                entry_point: "main",
                targets: &[sc_desc.format.into()], // &[wgpu::ColorTargetState {
                   //format: sc_desc.format,
                   //alpha_blend: wgpu::BlendState::REPLACE,
                   //color_blend: wgpu::BlendState::REPLACE, //REPLACE,
                   //write_mask: wgpu::ColorWrite::COLOR,
                },
            ),
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
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            None
        )
    }
}
