use crate::misc::create_vb_descriptor;
use crate::texture as jaankaup;
use crate::camera::{Camera};
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct VVVVNNNN {
//     pub position: [f32 ; 4],
//     pub normal: [f32 ; 4],
// }

//// #[derive(Clone)]
//// pub struct BasicRenderDescriptor {
////     vertex_descriptors: Vec<wgpu::VertexFormat>,
////     primitive_topology: wgpu::PrimitiveTopology,
////     number_of_samplers2D: u32,
////     camera_visibility: Option<wgpu::ShaderStage>,
////     has_depth_buffer: bool,
//// }
//// 
//// pub struct BasicRender {
////     pipeline: wgpu::RenderPipeline,
////     bing_group_layout: wgpu::BindGroupLayout,
////     render_descriptors: BasicRenderDescriptor,
//// }
//// 
//// 
//// // struct Entry<'a> {
//// //     group_id: u32,
//// //     entry: wgpu::BindGroupEntry<'a>,
//// // }
//// 
//// impl BasicRender {
//// 
////     pub fn init(device: &wgpu::Device,
////                 sc_desc: &wgpu::SwapChainDescriptor,
////                 render_descriptor: &BasicRenderDescriptor,
////                 vs_module: &wgpu::ShaderModule,
////                 fs_module: &wgpu::ShaderModule) -> Self {
//// 
////         let bind_group_layout = BasicRender::get_bind_group_layout(&device, &render_descriptor);
////         let pipeline = BasicRender::create_pipeline(
////             &device, 
////             &sc_desc,
////             &vs_module,
////             &fs_module,
////             &render_descriptor
////         );
//// 
////         Self {
////             pipeline: pipeline,
////             bing_group_layout: bind_group_layout,
////             render_descriptors: render_descriptor.clone(),
////         }
////     }
//// 
////     /// Create a bind groups.
////     //pub fn create_bind_group(&self, device: &wgpu::Device, textures: &Option<Vec<&jaankaup::Texture>>, camera: Option<Camera>) -> wgpu::BindGroup {
////     //pub fn create_bind_group(&self, device: &wgpu::Device, textures: &Option<Vec<&jaankaup::Texture>>, camera: Option<&Camera>) {
//// 
////     //    let joo = textures.as_ref();
////     //    match joo {
////     //        Some(tex) => log::info!("
////     //    }
////         // let bind_group_layout = BasicRender::get_bind_group_layout(&device, &self.render_descriptor);
//// 
////         // // Create bindings.
////         // let bind_group_entry0 = wgpu::BindGroupEntry {
////         //         binding: 0,
////         //         resource: wgpu::BindingResource::TextureView(&texture.view),
////         // };
//// 
////         // let bind_group_entry1 = wgpu::BindGroupEntry {
////         //         binding: 1,
////         //         resource: wgpu::BindingResource::Sampler(&texture.sampler),
////         // };
//// 
////         // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
////         //     layout: &bind_group_layout,
////         //     entries: &[bind_group_entry0, bind_group_entry1],
////         //     label: None,
////         // });
//// 
////         // bind_group
////     //}
//// 
////     /// Creates a basic render pipeline with an optional camera uniform and
////     /// number_of_samplers amount of bindable textures. 
////     pub fn create_pipeline(device: &wgpu::Device,
////                            sc_desc: &wgpu::SwapChainDescriptor,
////                            vs_module: &wgpu::ShaderModule,
////                            fs_module: &wgpu::ShaderModule,
////                            render_descriptor: &BasicRenderDescriptor,
////                            ) -> wgpu::RenderPipeline {
////     
////         assert!(render_descriptor.vertex_descriptors.len() > 0, "There must be at least one render descriptors.");
////     
////         // Create stride and vertex attribute descriptors.
////         let (stride, vb_desc) = create_vb_descriptor(&render_descriptor.vertex_descriptors);
////             
////         // Bind group layout for pipeline.
////         let bind_group_layout = BasicRender::get_bind_group_layout(&device, &render_descriptor);
////         
////         // Create pipeline layout.
////         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
////             label: Some("Basic_renderin_pipeline_layout"),
////             bind_group_layouts: &[&bind_group_layout],
////             push_constant_ranges: &[],
////         });
//// 
////         // Create the pipeline.
////         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
////             label: Some("Basic_render"),
////             layout: Some(&pipeline_layout),
////             vertex_stage: wgpu::ProgrammableStageDescriptor {
////                 module: &vs_module,
////                 entry_point: "main",
////             },
////             fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
////                 module: &fs_module,
////                 entry_point: "main",
////             }),
////             rasterization_state: Some(wgpu::RasterizationStateDescriptor {
////                 front_face: wgpu::FrontFace::Ccw,
////                 cull_mode: wgpu::CullMode::Back,
////                 ..Default::default()
////             }),
////             primitive_topology: wgpu::PrimitiveTopology::TriangleList,
////             color_states: &[sc_desc.format.into()],
////             depth_stencil_state: match render_descriptor.has_depth_buffer {
////                     true => Some(wgpu::DepthStencilStateDescriptor {
////                                 format: wgpu::TextureFormat::Depth32Float,
////                                 depth_write_enabled: true,
////                                 depth_compare: wgpu::CompareFunction::Less,
////                                 stencil: wgpu::StencilStateDescriptor {
////                                     front: wgpu::StencilStateFaceDescriptor::IGNORE,
////                                     back: wgpu::StencilStateFaceDescriptor::IGNORE,
////                                     read_mask: 0,
////                                     write_mask: 0,
////                                 },
////                             }),
////                     false => None,
////             },
////             vertex_state: wgpu::VertexStateDescriptor {
////                 index_format: None, //Some(wgpu::IndexFormat::Uint16),
////                 vertex_buffers: &[
////                     wgpu::VertexBufferDescriptor {
////                         stride: stride,
////                         step_mode: wgpu::InputStepMode::Vertex, 
////                         attributes: &vb_desc,
////                     }],
////             },
////             sample_count: 1,
////             sample_mask: !0,
////             alpha_to_coverage_enabled: false,
////         });
//// 
////         pipeline
////     }
//// 
////     /// Create layout for pipeline and bindgroups.
////     fn get_bind_group_layout(device: &wgpu::Device, render_descriptors: &BasicRenderDescriptor) -> wgpu::BindGroupLayout {
//// 
////         let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
//// 
////         // If there is a camera uniform...
////         if let Some(visibility) = render_descriptors.camera_visibility {
////             entries.push(wgpu::BindGroupLayoutEntry {
////                 binding: 0,
////                 visibility: visibility, //wgpu::ShaderStage::VERTEX,
////                 ty: wgpu::BindingType::Buffer {
////                     ty: wgpu::BufferBindingType::Uniform,
////                     has_dynamic_offset: false,
////                     min_binding_size: None,
////                 },
////                 count: None,
////             });
////         }
//// 
////         // Create 2d texture sampling entries.
////         for i in 0..render_descriptors.number_of_samplers2D {
////             entries.push(wgpu::BindGroupLayoutEntry {
////                 binding: 0,
////                 visibility: wgpu::ShaderStage::FRAGMENT,
////                 ty: wgpu::BindingType::Texture {
////                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
////                     view_dimension: wgpu::TextureViewDimension::D2,
////                     multisampled: false,
////                 },
////                 count: None,
////             });
////             entries.push(wgpu::BindGroupLayoutEntry {
////                 binding: 1,
////                 visibility: wgpu::ShaderStage::FRAGMENT,
////                 ty: wgpu::BindingType::Sampler {
////                     filtering: true,
////                     comparison: false,
////                 },
////                 count: None,
////             });
////         }
//// 
////         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
////             entries: &entries,
////             label: Some("Basic bind_group layout"),
////         })
////     }
//// }
