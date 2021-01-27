use crate::misc::create_vb_descriptor;
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct VVVVNNNN {
//     pub position: [f32 ; 4],
//     pub normal: [f32 ; 4],
// }

pub struct BasicRenderDescriptor {
    vertex_descriptors: Vec<wgpu::VertexFormat>,
    primitive_topology: wgpu::PrimitiveTopology,
    number_of_samplers2D: u32,
    has_camera_uniform: bool,
}

pub struct BasicRender {
    pipeline: wgpu::RenderPipeline,
    render_descriptors: BasicRenderDescriptor,
}

impl BasicRender {

    /// Creates a basic render pipeline with an optional camera uniform and
    /// number_of_samplers amount of bindable textures. 
    //pub fn create_pipeline(device: &wgpu::Device,
    //                       sc_desc: &wgpu::SwapChainDescriptor,
    //                       vert: &wgpu::ShaderModule,
    //                       frag: &wgpu::ShaderModule,
    //                       render_descriptor: &BasicRenderDescriptor,
    //                       depth_buffer: Option<wgpu::Buffer>, 
    //                       ) -> wgpu::RenderPipeline {
    //
    //    assert!(render_descriptor.vertex_descriptors.len() > 0, "There must be at least one render descriptors.");
    //
    //    // Create stride and vertex attribute descriptors.
    //    let (stride, vb_desc) = create_vb_descriptor(&vertex_descriptors);
    //        
    //    // Bind group layout for pipeline.
    //    let bind_group_layout = BasicRender::get_bind_group_layout(&device);
    //    
    //    // Create pipeline layout.
    //    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //        label: Some("two_triangles_pipeline_layout"),
    //        bind_group_layouts: &[&bind_group_layout],
    //        push_constant_ranges: &[],
    //    });
    //}

    /// Create layout for pipeline and bindgroups.
    fn get_bind_group_layout(device: &wgpu::Device, info: &BasicRenderDescriptor) -> wgpu::BindGroupLayout {

        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();

        // If the is a camera uniform...
        if info.has_camera_uniform {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        // Create 2d texture sampling entries.
        for i in 0..info.number_of_samplers2D {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: true,
                    comparison: false,
                },
                count: None,
            });
        }



         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
             entries: &entries, //&entries.iter().collect::<Vec<_>>(),
             label: Some("Basic bind_group layout"),
         })
    }
}

