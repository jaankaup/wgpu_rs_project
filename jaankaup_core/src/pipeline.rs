//#[derive(Clone, Copy)]
//struct ShaderModuleInfo {
//    name: &'static str,
//    source_file: &'static str,
//    _stage: &'static str, // TODO: remove? 
//}

enum Resource {
    TextureView(&'static str),
    TextureSampler(&'static str),
    Buffer(&'static str),
}

/// A struct that holds information for one draw call.
pub struct VertexBufferInfo {
    vertex_buffer_name: String,
    _index_buffer: option<string>,
    start_index: u32,
    end_index: u32,
    instances: u32,
}

/// A struct for a single render pass.
pub struct RenderPass {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
}

/// A struct for a single compute pass.
struct ComputePass {
    pipeline: wgpu::ComputePipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    dispatch_x: u32,
    dispatch_y: u32,
    dispatch_z: u32,
}

impl RenderPass {
    /// Execute current render pass. TODO: multisampled doesn't work as expected. 
    fn execute(&self,
               encoder: &mut wgpu::CommandEncoder,
               frame: &wgpu::SwapChainTexture,
               multisampled_framebuffer: &wgpu::TextureView,
               textures: &HashMap<String, Texture>,
               buffers: &HashMap<String, Buffer>,
               vertex_buffer_info: &VertexBufferInfo,
               sample_count: u32,
               clear: bool) {

            let multi_sampled = multisampled(sample_count);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: Borrowed(&[
                    wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: match multi_sampled { false => &frame.view, true => &multisampled_framebuffer, },
                            resolve_target: match multi_sampled { false => None, true => Some(&frame.view), },
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
                ]),
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &textures.get(TEXTURES.depth.name).unwrap().view,
                    depth_ops: Some(wgpu::Operations {
                            load: match clear { true => wgpu::LoadOp::Clear(1.0), false => wgpu::LoadOp::Load }, 
                            store: true,
                    }),
                    stencil_ops: None,
                    }),
            });

            render_pass.set_pipeline(&self.pipeline);

            // Set bind groups. TODO: bundles.
            for (e, bgs) in self.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(e as u32, &bgs, &[]);
            }

            // Set vertex buffer.
            render_pass.set_vertex_buffer(
                0,
                buffers.get(&vertex_buffer_info.vertex_buffer_name).unwrap().buffer.slice(..)
            );

            // TODO: handle index buffer.

            // Draw.
            render_pass.draw(vertex_buffer_info.start_index..vertex_buffer_info.end_index, 0..vertex_buffer_info.instances);
    }
}

impl ComputePass {

    fn execute(&self, encoder: &mut wgpu::CommandEncoder) {

        let mut ray_pass = encoder.begin_compute_pass();
        ray_pass.set_pipeline(&self.pipeline);
        for (e, bgs) in self.bind_groups.iter().enumerate() {
            ray_pass.set_bind_group(e as u32, &bgs, &[]);
        }
        ray_pass.dispatch(self.dispatch_x, self.dispatch_y, self.dispatch_z);
    }
}

/// A struct that keep information about one binding.
struct BindGroupInfo {
    binding: u32,
    visibility: wgpu::ShaderStage,
    resource: Resource, 
    binding_type: wgpu::BindingType,
}