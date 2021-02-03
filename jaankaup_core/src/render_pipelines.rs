pub trait LayoutEntry {
    fn create_bind_group_layouts(device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout>;
}

/// Creates the BindGroupLayout.
//fn create_bind_group_layout(device: &wgpu::Device, entries: &Vec<NumberedEntry>) -> Vec<wgpu::BindGroupLayout> {
fn create_bind_group_layout(device: &wgpu::Device, entries: &wgpu::BindGroupLayoutEntry) {

    //for entry in entries.iter() {
    //    let sub_entries = layout_descriptors.entry(entry.set).or_insert(Vec::new());
    //    sub_entries.push(entry.entry.clone());
    //}
}

pub fn create_bind_groups(entry_layouts: &Vec<Vec<wgpu::BindGroupLayoutEntry>>, bindings: &Vec<Vec<&wgpu::BindingResource>>) {

    //let result: Vec<Vec<wgpu::BindGroupEntry>> = entry_layouts

    assert!(entry_layouts.len() == bindings.len(), "The length of entry_layouts mismatches with bindings. {} != {}", 
        entry_layouts.len(), bindings.len());

    let result: Vec<Vec<wgpu::BindGroup>> = Vec::new();

    for i in 0..entry_layouts.len() {

        assert!(entry_layouts[i].len() == bindings[i].len(), "The length of entry_layouts[{}] mismatches with bindings[{}]. {} != {}", 
            i, i, entry_layouts[i].len(), bindings[i].len());

        let mut inner_group: Vec<wgpu::BindGroupEntry> = Vec::new();

        for j in 0..entry_layouts[i].len() {

            match entry_layouts[i][j] {
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Buffer { .. },
                     count: _,
                 } => {

                     match bindings[i][j] {
                             wgpu::BindingResource::Buffer { .. } => { inner_group.push(
                                 wgpu::BindGroupEntry {
                                     binding: j as u32,
                                     resource: bindings[i][j].clone(),
                                 });
                             },
                             _ => panic!("Entry layout and binding mismatch (Buffer).")
                         }
                     },
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Texture { .. }, 
                     count: _,
                 } => {
                     match bindings[i][j] {
                             wgpu::BindingResource::TextureView(_) => { inner_group.push(
                                 wgpu::BindGroupEntry {
                                     binding: j as u32,
                                     resource: bindings[i][j].clone(),
                                 });
                             },
                             _ => panic!("Entry layout and binding mismatch (Texture).")
                         }
                     },
                 wgpu::BindGroupLayoutEntry {
                     binding: _,
                     visibility: _,
                     ty: wgpu::BindingType::Sampler { .. }, 
                     count: _,
                 } => {
                     match bindings[i][j] {
                             wgpu::BindingResource::Sampler(_) => { inner_group.push(
                                 wgpu::BindGroupEntry {
                                     binding: j as u32,
                                     resource: bindings[i][j].clone(),
                                 });
                             },
                             _ => panic!("Entry layout and binding mismatch (Sampler).")
                         }
                     },
                 _ => { unimplemented!("Now implemented yet or error.") },
        } // j
    } // i
}

//                    match bindings[i][j] {
//                            wgpu::BindingResource::Texture { .. } => { inner_group.push(
//                                wgpu::BindGroupEntry {
//                                    binding: j as u32,
//                                    resource: bindings[i][j].clone(),
//                                });
//                            },
//                            _ => panic!("Entry layout and binding mismatch.")
//                        }
//                    }
//                _ => {}
//            }
//                    log::info!("Juuuuu on Bufferi"); },
//
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::Sampler  { .. },
//                    count: _,
//                } => { log::info!("No sehä on sampleri!"); },
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::Texture  { .. },
//                    count: _,
//                } => { log::info!("No vieha että, tekstuuri!"); },
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::StorageTexture { .. },
//                    count: _,
//                } => { log::info!("Yes, storage tekstuuri!"); },
//                _ => { log::info!("joku muu"); },
//           }
        }



//    let result: Vec<Vec<&str>> = entry_layouts
//        .iter().zip(bindings.iter())
//        .map(|set| if let Some(s, b) {
//                        s.iter().zip(b.iter())
//                        .map(|pah| if let Some(a, b) {
//                            "jeejee"
//                        }
//                        else {
//                            "ei jeejee"
//                        }
//                   }
//                   else { panic!("Different amount of BindGropuEntries and BindingResources") }
//
//        
//        
//        set.iter().map(|entry| 
//           match entry {
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::Buffer { .. },
//                    count: _,
//                } => "Juuuuu on Bufferi",
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::Sampler  { .. },
//                    count: _,
//                } => "No sehä on sampleri!",
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::Texture  { .. },
//                    count: _,
//                } => "No vieha että, tekstuuri!",
//                wgpu::BindGroupLayoutEntry {
//                    binding: _,
//                    visibility: _,
//                    ty: wgpu::BindingType::StorageTexture { .. },
//                    count: _,
//                } => "Yes, storage tekstuuri!",
//                _ => "joku muu",
//           }
//        ).collect())
//        .collect();
////                        ty: wgpu::BindingType::Sampler { .. } => "Samplerihan se!",
////                        ty: wgpu::BindingType::Texture { .. } => "Texturihan se!",
////                        ty: wgpu::BindingType::StorageTexture { .. } => "Storage tekstuurihan se!",
////
//    for elem in result.iter() {
//        for e in elem.iter() {
//            log::info!("{}", e);
//    }}
//   //for (i, elem) in entry_layouts.enumerate() {
//
//   //}

#[derive(Clone)]
pub struct BasicRenderDescriptor {
    vertex_descriptors: Vec<wgpu::VertexFormat>,
    primitive_topology: wgpu::PrimitiveTopology,
    number_of_samplers2D: u32,
    camera_visibility: Option<wgpu::ShaderStage>,
    has_depth_buffer: bool,
}

pub struct NumberedEntry {
    set: u32,
    entry: wgpu::BindGroupLayoutEntry,
}

pub struct TestLayoutEntry {

    /// BindGroupEntries in descending set number order.
    /// This is the interface to the pipeline. Create actual binding using 
    /// wgpu::BindGroupEntry and wgpu::BindGroupDescriptor.
    pub entry_layout: Vec<Vec<wgpu::BindGroupLayoutEntry>>, 
}

impl TestLayoutEntry {
    pub fn init() -> Self {

        /// Define all bind grout entries for pipeline and bind groups.
        let entry_layouts = vec![
                // Set 0
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
                    }] // Set 0
            ];

        Self {
            entry_layout: entry_layouts,
        }
    }
}
    
//    pub fn create_bind_groups(&self,
//                              device: &wgpu::Device,
//                              texture: &wgpu::Texture,
//                              texture_view: &wgpu::TextureView
//                             ) -> Vec<wgpu::BindGroup> { 
//
//        let bind_group_layout = TwoTriangles::get_bind_group_layout(&device);
//
//        // Create bindings.
//        let bind_group_entry0 = wgpu::BindGroupEntry {
//                binding: 0,
//                resource: wgpu::BindingResource::TextureView(&texture.view),
//        };
//
//        let bind_group_entry1 = wgpu::BindGroupEntry {
//                binding: 1,
//                resource: wgpu::BindingResource::Sampler(&texture.sampler),
//        };
//
//        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//            layout: &bind_group_layout,
//            entries: &[bind_group_entry0, bind_group_entry1],
//            label: None,
//        });
//
//        bind_group
//    }
//}
//
//impl LayoutEntry for TestLayoutEntry {
//    fn create_bind_group_layouts(
//        &self,
//        device: &wgpu::Device,
//        entries: &Vec<wgpu::BingGroupLayoutEntry>
//        ) -> Vec<wgpu::BindGroupLayout> {
//        
//        let bind_group_layout =
//            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                entries: &entries, 
//                label: Some("two_triangles_bind_group_layout"),
//        });
//        bind_group_layout
//    }
//}

//pub fn create_bind_group<T>
