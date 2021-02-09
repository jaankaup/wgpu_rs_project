use core::ops::Range;
use crate::misc::create_vb_descriptor;
use crate::texture as jaankaup;

pub trait LayoutEntry {
    fn create_bind_group_layouts(device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout>;
}

pub struct MyTestPipeline {

}

pub fn draw(encoder: &mut wgpu::CommandEncoder,
            frame: &wgpu::SwapChainTexture,
            depth_texture: &jaankaup::Texture,
            bind_groups: &Vec<wgpu::BindGroup>,
            pipeline: &wgpu::RenderPipeline,
            draw_buffer: &wgpu::Buffer,
            range: Range<u32>, 
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
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
    binding_resources: &Vec<Vec<wgpu::BindingResource>>) -> bool {

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

// pub struct NumberedEntry {
//     set: u32,
//     entry: wgpu::BindGroupLayoutEntry,
// }

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
                sc_desc: &wgpu::SwapChainDescriptor,               
                vs_module: &wgpu::ShaderModule,
                fs_module: &wgpu::ShaderModule,
                ) -> Self {

        // Define all bind grout entries for pipeline and bind groups.
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
                // Set 1
                vec![wgpu::BindGroupLayoutEntry {
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    }
                ] // Set 1
        ];

        let bind_group_layouts = create_bind_group_layouts(&device, &layout_entries);

        // Create stride and vertex attribute descriptors.
        let (stride, attributes) =  create_vb_descriptor(
            &vec![wgpu::VertexFormat::Float4, wgpu::VertexFormat::Float4]
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Front,
                polygon_mode: wgpu::PolygonMode::Fill,
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
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                   format: sc_desc.format,
                   alpha_blend: wgpu::BlendState::REPLACE,
                   color_blend: wgpu::BlendState::REPLACE,
                   write_mask: wgpu::ColorWrite::COLOR,
                }],
            }),
        });

        Self {
            layout_entries: layout_entries,
            bind_group_layouts: bind_group_layouts,
            pipeline: pipeline,
        }
    }
}
