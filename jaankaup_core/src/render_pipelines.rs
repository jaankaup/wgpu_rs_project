use core::ops::Range;
use crate::misc::create_vb_descriptor;
use crate::texture as jaankaup;

//trait RenderPipelineInfo {
//    pub fn get_render_pipeline(&self) -> Option<&wgpu::RenderPipeline>;
//    pub 
//    pub layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, // !! 
//    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>, // create and store. Used for pipeline and bind groups.
//    pub pipeline: wgpu::RenderPipeline,
//
//}

pub fn draw(encoder: &mut wgpu::CommandEncoder,
            //frame: &wgpu::SurfaceFrame,
            view: &wgpu::TextureView,
            depth_texture: &jaankaup::Texture,
            bind_groups: &Vec<wgpu::BindGroup>,
            pipeline: &wgpu::RenderPipeline,
            draw_buffer: &wgpu::Buffer,
            range: Range<u32>, 
            clear: bool) {

        // println!("yhhyyy!");
        // let view = frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // println!("yhhyyy2!");
        let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("two_triangles_rendes_pass_descriptor"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                                view: &view,
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
                //depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    //attachment: &depth_texture.view,
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                            load: match clear { true => wgpu::LoadOp::Clear(1.0), false => wgpu::LoadOp::Load },
                            store: true,
                    }),
                    stencil_ops: None,
                    }),
            });

            render_pass.set_pipeline(&pipeline);
            // Set bind groups.
            for (e, bgs) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(e as u32, &bgs, &[]);
            }

            // Set vertex buffer.
            render_pass.set_vertex_buffer(
                0,
                draw_buffer.slice(..)
            );

            render_pass.draw(range, 0..1);
    }

/// 1. Create BindGroupLayouts from the BindGroupLayoutEntries.
/// BindGroupLayouts can be used to create pipelines and BindGroups.
pub fn create_bind_group_layouts(device: &wgpu::Device, layout_entries: &Vec<Vec<wgpu::BindGroupLayoutEntry>>) -> Vec<wgpu::BindGroupLayout> {

    let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();
    for e in layout_entries.iter() {
        bind_group_layouts.push(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &e,
                label: None,
            }
        ));
    }
    bind_group_layouts
}

/// A debug helper function to check that the given binding resources match the bind group layout.
/// Function will panic if the entries and binding resources mismatch.
pub fn check_correspondence(
    bind_group_layout_entries: &Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    binding_resources: &Vec<Vec<&wgpu::BindingResource>>) -> bool {

    // Check that the number of groups are the same.
    assert!(bind_group_layout_entries.len() == binding_resources.len(),
        "The length of bind_group_layout_entries mismatches with binding_resources. {} != {}", 
        bind_group_layout_entries.len(), binding_resources.len()
    );

    // Check that the number of entries match.
    for i in 0..bind_group_layout_entries.len() {

        assert!(bind_group_layout_entries[i].len() == binding_resources[i].len(),
            "The length of bind_group_layout_entries[{}] mismatches with binding_resources[{}]. {} != {}", 
            i, i, bind_group_layout_entries[i].len(), binding_resources[i].len());

        // Check the correspondence of entries and resourced.
        for j in 0..bind_group_layout_entries[i].len() {

            match bind_group_layout_entries[i][j] {
                 // If the entry is a buffer => resource must be a buffer.
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Buffer { .. },
                     count: _,
                 } => {
                     // Check if the corresponding resource is a buffer.
                     match binding_resources[i][j] {
                             wgpu::BindingResource::Buffer { .. } => { /* Ok. */ },                              
                             _ => panic!("Entry layout and binding mismatch at index [{}][{}] (Buffer).", i,j),
                     }
                 },
                 // If the entry is a texture => resource must be a texture.
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Texture { .. }, 
                     count: _,
                 } => {
                     match binding_resources[i][j] {
                             wgpu::BindingResource::TextureView(_) => { /* Ok. */ },
                             _ => panic!("Entry layout and binding mismatch at index [{}][{}] (Texture).", i, j),
                         }
                     },
                 // If the entry is a sampler => resource must be a sampler.
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Sampler { .. }, 
                     count: _,
                 } => {
                     match binding_resources[i][j] {
                             wgpu::BindingResource::Sampler(_) => { /* Ok. */ },
                             _ => panic!("Entry layout and binding mismatch (Sampler)."),
                         }
                     },
                 _ => { unimplemented!("Now implemented yet or something error.") },
            }
        }
    }
    true
}

/// Create bindgroups from bind group layout entries and binding resources.
/// This function assemes that binding resources correspond to bind group layout enties.
/// The correspondence can be checked with check_correspondence function.
pub fn create_bind_groups(device: &wgpu::Device, 
                          entry_layouts: &Vec<Vec<wgpu::BindGroupLayoutEntry>>,
                          bindings: &Vec<Vec<&wgpu::BindingResource>>)
                        -> Vec<wgpu::BindGroup> {

    // The created bindgroups.
    let mut result: Vec<wgpu::BindGroup> = Vec::new();

    // Add Binding resources to the bind group.
    for i in 0..entry_layouts.len() {

        let mut inner_group: Vec<wgpu::BindGroupEntry> = Vec::new();

        // Create the bind groups. TODO: this should be created only once (add to struct
        // attribute).
        let layouts = create_bind_group_layouts(&device, &entry_layouts);

        for j in 0..entry_layouts[i].len() {

            // Create bind group entry from rresource.
            inner_group.push(
                wgpu::BindGroupEntry {
                    binding: j as u32,
                    resource: bindings[i][j].clone(),
                }
            );

            // If all bind group entries has been created, create BindGroup.
            if j == entry_layouts[i].len() - 1 {
                result.push(device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layouts[i],
                        entries: &inner_group,
                    })
                );
            }
        } // j
    } // i
    result
}

#[derive(Clone)]
pub struct RenderDescriptor {
    vertex_descriptors: Vec<wgpu::VertexFormat>,
    primitive_topology: wgpu::PrimitiveTopology,
    has_depth_buffer: bool,
}

pub struct TestLayoutEntry {

    /// BindGroupEntries in descending set number order.
    /// This is the interface to the pipeline. Create actual binding using 
    /// wgpu::BindGroupEntry and wgpu::BindGroupDescriptor.
    pub layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>, 
    pub pipeline: wgpu::RenderPipeline,
}

impl TestLayoutEntry {
    pub fn init(device: &wgpu::Device,
                sc_desc: &wgpu::SurfaceConfiguration,               
                wgsl_module: &wgpu::ShaderModule,
                // vs_module: &wgpu::ShaderModule,
                // fs_module: &wgpu::ShaderModule,
                ) -> Self {

        // Define all bind grout entries for pipeline and bind groups.
        log::info!("TestLayoutEntry::init");
        let layout_entries = vec![
                // Set 0
                vec![wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            },
                        count: None,
                    },
                ],
                // Set 1
                vec![wgpu::BindGroupLayoutEntry {
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    }
                ] // Set 1
        ];

        // Create stride and vertex attribute descriptors.
        let (stride, attributes) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]
        );

        let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();

        // Create bind group layout descriptors. This is used to create bind group layout.
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
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(), // &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wgsl_module,
                entry_point: "vs_main",
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
                cull_mode: Some(wgpu::Face::Front),
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
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &wgsl_module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    blend: None, //Some(wgpu::BlendState {
                           //     color: wgpu::BlendComponent {
                           //          src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                           //          dst_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                           //          operation: wgpu::BlendOperation::Max, 
                           //     },
                           //     alpha: wgpu::BlendComponent {
                           //          src_factor: wgpu::BlendFactor::SrcAlpha,
                           //          dst_factor: wgpu::BlendFactor::One,
                           //          operation: wgpu::BlendOperation::Add, 
                           //     },
                           // }),
                    // alpha_blend: wgpu::BlendState::REPLACE,
                    // color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrites::COLOR,
                }],
            }),
        });
        log::info!("TestLayoutEntry::init == OK");

        Self {
            layout_entries: layout_entries,
            bind_group_layouts: bind_group_layouts,
            pipeline: pipeline,
        }
    }
}
