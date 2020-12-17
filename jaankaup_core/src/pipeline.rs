//use crate::pipeline::Resource::Buffer;
use std::borrow::Cow::Borrowed;
//use std::borrow::Cow::Borrowed;
use std::collections::HashMap;
use crate::texture::*;
use crate::buffer::Buffer as JBuffer;
use crate::misc::multisampled;
use crate::shader::ShaderModule;

//#[derive(Clone, Copy)]
//struct ShaderModuleInfo {
//    name: &'static str,
//    source_file: &'static str,
//    _stage: &'static str, // TODO: remove? 
//}

/// A struct that holds information for one draw call.
pub struct VertexBufferInfo {
    vertex_buffer_name: String,
    _index_buffer: Option<String>,
    start_index: u32,
    end_index: u32,
    instances: u32,
}

pub enum Resource {
    TextureView(&'static str),
    TextureSampler(&'static str),
    Buffer(&'static str),
}

pub struct RenderPipelineInfo<'a> {
    vertex_shader: String, //ShaderModule, //ShaderModuleInfo,
    fragment_shader: Option<&'a str>,
    bind_groups: Vec<Vec<BindGroupInfo>>,
    input_formats: Vec<(wgpu::VertexFormat, u64)>,
}

pub struct ComputePipelineInfo {
    compute_shader: String,
    bind_groups: Vec<Vec<BindGroupInfo>>,
}

/// A struct for a single render pass.
pub struct RenderPass {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
}

/// A struct for a single compute pass.
pub struct ComputePass {
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
               buffers: &HashMap<String, JBuffer>,
               vertex_buffer_info: &VertexBufferInfo,
               sample_count: u32,
               clear: bool) {

            let multi_sampled = multisampled(sample_count);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
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
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &textures.get("depth_texture").unwrap().view, // TODO: depth texture from function parameter?
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
pub struct BindGroupInfo {
    binding: u32,
    visibility: wgpu::ShaderStage,
    resource: Resource, 
    binding_type: wgpu::BindingType,
}

pub fn create_render_pipeline_and_bind_groups(device: &wgpu::Device,
                                   sc_desc: &wgpu::SwapChainDescriptor,
                                   shaders: &HashMap<String, wgpu::ShaderModule>,
                                   textures: &HashMap<String, Texture>,
                                   buffers: &HashMap<String, JBuffer>,
                                   rpi: &RenderPipelineInfo,
                                   primitive_topology: &wgpu::PrimitiveTopology,
                                   sample_count: u32)
    -> (Vec<wgpu::BindGroup>, wgpu::RenderPipeline) {

    print!("    * Creating bind groups ... ");

    let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();
    let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();

    // Loop over all bind_groups.
    for b_group in rpi.bind_groups.iter() {

        let layout_entries: Vec<wgpu::BindGroupLayoutEntry>
            = b_group.into_iter().map(|x| wgpu::BindGroupLayoutEntry {
                binding: x.binding,
                visibility: x.visibility,
                ty: x.binding_type.clone(),
                count: None,
              }).collect();


           device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
               entries: &Borrowed(&layout_entries),
               label: None,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &Borrowed(&layout_entries),
                label: None,
            });

        let bindings: Vec<wgpu::BindGroupEntry>
            = b_group.into_iter().map(|x| wgpu::BindGroupEntry {
                binding: x.binding,
                resource: match x.resource {
                        Resource::TextureView(tw) =>
                            wgpu::BindingResource::TextureView(&textures.get(tw).expect(&format!("Failed to get texture {}.", tw)).view),
                        Resource::TextureSampler(ts) =>
                            wgpu::BindingResource::Sampler(&textures.get(ts).expect(&format!("Failed to get texture {}.", ts)).sampler),
                        Resource::Buffer(b) =>
                            buffers.get(b).expect(&format!("Failed to get buffer {}.", b)).buffer.as_entire_binding(),
                            //wgpu::BindingResource::Buffer(buffers.get(b).expect(&format!("Failed to get buffer {}.", b)).buffer.slice(..)),
                }
            }).collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &Borrowed(&bindings),
            label: None,
        });

        bind_group_layouts.push(texture_bind_group_layout);
        bind_groups.push(bind_group);
    }

    println!(" OK'");

    print!("    * Creating pipeline ... ");

      let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
          label: None,
          bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
          push_constant_ranges: &[],
      });

      // Crete vertex attributes.
      let mut stride: u64 = 0;
      let mut vertex_attributes: Vec<wgpu::VertexAttributeDescriptor> = Vec::new();
      println!("rpi.input.formats.len() == {}", rpi.input_formats.len());
      for i in 0..rpi.input_formats.len() {
          vertex_attributes.push(
              wgpu::VertexAttributeDescriptor {
                  format: rpi.input_formats[i].0,
                  offset: stride,
                  shader_location: i as u32,
              }
          );
          stride = stride + rpi.input_formats[i].1;
          println!("stride {} :: {}", i, stride);
      }

      let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &shaders.get(&rpi.vertex_shader).expect(&format!("Failed to get vertex shader {}.", &rpi.vertex_shader)),
            entry_point: &"main",
        },
        fragment_stage: match rpi.fragment_shader {
            None => None,
            Some(s)    => Some(wgpu::ProgrammableStageDescriptor {
                            module: &shaders.get(s).expect(&format!("Failed to fragment shader {}.", s)),
                            entry_point: &"main",
                    }),
        },
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None, // Back
            ..Default::default()
        }),
        primitive_topology: *primitive_topology, //wgpu::PrimitiveTopology::TriangleList,
        color_states: &[
            wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            },
        ],
        //depth_stencil_state: None,
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // Less
            stencil: wgpu::StencilStateDescriptor {
                front: wgpu::StencilStateFaceDescriptor::IGNORE,
                back: wgpu::StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            //stencil_read_only: false,
        }),
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: stride,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &Borrowed(&vertex_attributes),
            }],
        },
        sample_count: sample_count,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
      });


    println!(" OK'");
    (bind_groups, render_pipeline)
}

pub fn create_compute_pipeline_and_bind_groups(device: &wgpu::Device,
                                           shaders: &HashMap<String, wgpu::ShaderModule>,
                                           textures: &HashMap<String, Texture>,
                                           buffers: &HashMap<String, JBuffer>,
                                           rpi: &ComputePipelineInfo)
    -> (Vec<wgpu::BindGroup>, wgpu::ComputePipeline) {

    print!("    * Creating compute bind groups ... ");

    let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();
    let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();

    // Loop over all bind_groups.
    for b_group in rpi.bind_groups.iter() {

        let layout_entries: Vec<wgpu::BindGroupLayoutEntry>
            = b_group.into_iter().map(|x| wgpu::BindGroupLayoutEntry {
                binding: x.binding,
                visibility: x.visibility,
                ty: x.binding_type.clone(),
                count: None, // TODO: check this out later?
              }).collect();

        let texture_bind_group_layout =
           device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
               entries: &Borrowed(&layout_entries),
               label: None,
        });

        let bindings: Vec<wgpu::BindGroupEntry>
            = b_group.into_iter().map(|x| wgpu::BindGroupEntry {
                binding: x.binding,
                resource: match x.resource {
                        Resource::TextureView(tw) =>
                            wgpu::BindingResource::TextureView(&textures.get(tw).expect(&format!("Failed to get texture {}.", tw)).view),
                        Resource::TextureSampler(ts) =>
                            wgpu::BindingResource::Sampler(&textures.get(ts).expect(&format!("Failed to get texture {}.", ts)).sampler),
                        Resource::Buffer(b) =>
                            buffers.get(b).expect(&format!("Failed to get buffer {}.", b)).buffer.as_entire_binding(),
                            //wgpu::BindingResource::Buffer(buffers.get(b).expect(&format!("Failed to get buffer {}.", b)).buffer.slice(..)),
                }
            }).collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &Borrowed(&bindings),
            label: None,
        });

        bind_group_layouts.push(texture_bind_group_layout);
        bind_groups.push(bind_group);
    }

    println!(" OK'");

    print!("    * Creating compute pipeline ... ");

      let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
          label: None, // TODO: add label
          bind_group_layouts: &Borrowed(&bind_group_layouts.iter().collect::<Vec<_>>()),
          push_constant_ranges: &[],
      });

      let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: None, // TODO: add label
          layout: Some(&compute_pipeline_layout),
          compute_stage: wgpu::ProgrammableStageDescriptor {
              module: &shaders.get(&rpi.compute_shader).unwrap(),
              entry_point: &"main",
          },
      });


    println!(" OK'");
    (bind_groups, compute_pipeline)
}
