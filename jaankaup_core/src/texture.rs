use crate::misc::Convert2Vec;
use std::num::NonZeroU32;
use bytemuck::Pod;
//use std::mem;

/// All possible texture types. TODO: Are these necessery?
pub enum TextureType {
    Diffuse,
    Depth,
}

/// Texture.
pub struct Texture {
    pub texture_type: TextureType, 
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    width: u32,
    height: u32,
    depth: u32,
}

impl Texture {

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration, label: Option<&str>) -> Self {

        let width = sc_desc.width; 
        let height = sc_desc.height; 
        let depth = 1; 

        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: depth,
        };
        let desc = wgpu::TextureDescriptor {
            label: label,
            size,
            // array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, // TODO: SAMPLED?
            //    | wgpu::TextureUsages::SAMPLED
            //    | wgpu::TextureUsages::COPY_SRC,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::Less),
            //compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let texture_type = TextureType::Depth;

        Self { texture_type, texture, view, sampler, width, height, depth }
    }

    /// Creates a texture from a sequency of bytes (expects bytes to be in png format in rgb). Now
    /// its adding automaticallhy an alpha value of
    /// 255 to the image. TODO: check if aplha value already exists. TODO: allow a texture to been
    /// created from non png data.
    pub fn create_from_bytes(queue: &wgpu::Queue, device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration, sample_count : u32, bytes: &[u8], label: Option<&str>) -> Self {

        //let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            //compare: Some(wgpu::CompareFunction::Always),
            compare: None, // Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let png = std::io::Cursor::new(bytes);
        let decoder = png::Decoder::new(png);
        let (info, mut reader) = decoder.read_info().expect("Can't read info!");
        let width = info.width;
        let height = info.height;
        let bits_per_pixel = info.color_type.samples() as u32;

        // log::info!("widht :: {}", width);
        // log::info!("height :: {}", height);
        // log::info!("bits_per_pixel :: {}", bits_per_pixel);

        if !(bits_per_pixel == 3 || bits_per_pixel == 4) {
            panic!("Bits per pixel must be 3 or 4. Bits per pixel == {}", bits_per_pixel); 
        }

        let mut buffer: Vec<u8> = vec![0; (info.width * bits_per_pixel * info.height) as usize ];
        reader.next_frame(&mut buffer).unwrap(); //expect("Can't read next frame.");

        // TODO: check the size of the image.

        let mut temp: Vec<u8> = Vec::new();

        // The png has only rgb components. Add the alpha component to each texel. 
        if bits_per_pixel == 3 {
            for i in 0..buffer.len()/3 {
                let offset = i*3;
                let red: u8 = buffer[offset];
                let green: u8 = buffer[offset+1];
                let blue: u8 = buffer[offset+2];
                temp.push(blue); // blue
                temp.push(green); // green
                temp.push(red); // red
                temp.push(255); // alpha
            }
        }

        // log::info!("{}", temp.len());

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format, // wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        // log::info!("Writing texture.");
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            match bits_per_pixel {
                3 => &temp,
                4 => &buffer,
                _ => panic!("Bits size of {} is not supported", bits_per_pixel),
            },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(width * 4).unwrap()), // now only 4 bits per pixel is supported,
                rows_per_image: Some(NonZeroU32::new(height).unwrap()),
            },
            texture_extent,
            //wgpu::Extent3d::default(), //texture_extent,
        );
        // log::info!("Writing texture, OK.");

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(sc_desc.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            //aspect: wgpu::TextureAspect::default(),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: std::num::NonZeroU32::new(1),
            //mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let texture_type = TextureType::Diffuse;

        let width = texture_extent.width;
        let height = texture_extent.height;
        let depth = texture_extent.depth_or_array_layers;

        Self {

            texture_type, 
            texture,
            view,
            sampler,
            width,
            height,
            depth,
        }
    }

    //++ pub fn create_storage2d(device: &wgpu::Device,
    //++                         sc_desc: &wgpu::SwapChainDescriptor,
    //++                         width: u32,
    //++                         height: u32) -> Self {

    //++     let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
    //++         address_mode_u: wgpu::AddressMode::MirrorRepeat,
    //++         address_mode_v: wgpu::AddressMode::MirrorRepeat,
    //++         address_mode_w: wgpu::AddressMode::MirrorRepeat,
    //++         mag_filter: wgpu::FilterMode::Linear,
    //++         min_filter: wgpu::FilterMode::Linear,
    //++         mipmap_filter: wgpu::FilterMode::Linear,
    //++         lod_min_clamp: -100.0,
    //++         lod_max_clamp: 100.0,
    //++         compare: None, //Some(wgpu::CompareFunction::Always),
    //++         ..Default::default()
    //++     });

    //++     let texture_extent = wgpu::Extent3d {
    //++         width: width,
    //++         height: height,
    //++         depth_or_array_layers: 1,
    //++     };

    //++     let texture = device.create_texture(&wgpu::TextureDescriptor {
    //++         size: texture_extent,
    //++         mip_level_count: 1,
    //++         sample_count: 1,
    //++         dimension: wgpu::TextureDimension::D2,
    //++         format: sc_desc.format, //wgpu::TextureFormat::Rgba8UnormSrgb,
    //++         usage: wgpu::TextureUsages::TEXTURE_BINDING |
    //++                wgpu::TextureUsages::COPY_DST |
    //++                wgpu::TextureUsages::STORAGE_BINDING, // TODO: as function parameter
    //++         label: None,
    //++     });

    //++     let view = texture.create_view(&wgpu::TextureViewDescriptor {
    //++         label: None,
    //++         format: Some(sc_desc.format),// gpu::TextureFormat::Rgba8UnormSrgb,
    //++         dimension: Some(wgpu::TextureViewDimension::D2),
    //++         aspect: wgpu::TextureAspect::All,
    //++         base_mip_level: 0,
    //++         //level_count: std::num::NonZeroU32::new(1),
    //++         mip_level_count: std::num::NonZeroU32::new(1),
    //++         base_array_layer: 0,
    //++         array_layer_count: std::num::NonZeroU32::new(1),
    //++     });

    //++     let texture_type = TextureType::Diffuse;

    //++     let depth = 1;

    //++     Self {

    //++         texture_type, 
    //++         texture,
    //++         view,
    //++         sampler,
    //++         width,
    //++         height,
    //++         depth, 
    //++     }
    //++ }

    pub fn create_texture2d(device: &wgpu::Device,
                            sc_desc: &wgpu::SurfaceConfiguration,
                            sample_count: u32,
                            width: u32,
                            height: u32) -> Self {

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None, //Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format, //wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                   wgpu::TextureUsages::COPY_DST,
            label: None,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(sc_desc.format),// gpu::TextureFormat::Rgba8UnormSrgb,
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            //level_count: std::num::NonZeroU32::new(1),
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let texture_type = TextureType::Diffuse;

        let depth = 1;

        Self {

            texture_type, 
            texture,
            view,
            sampler,
            width,
            height,
            depth, 
        }
    }

    pub fn create_texture3d(device: &wgpu::Device, format: &wgpu::TextureFormat, width: u32, height: u32, depth: u32) -> Self {

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None, //Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: depth,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1, // this must always be 1
            dimension: wgpu::TextureDimension::D3,
            format: *format, //wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            label: None,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None, // TODO: add label to function parameter list
            format: Some(*format),// wgpu::TextureFormat::Rgba8UnormSrgb,
            dimension: Some(wgpu::TextureViewDimension::D3),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            //level_count: std::num::NonZeroU32::new(1),
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let texture_type = TextureType::Diffuse;

        Self {

            texture_type, 
            texture,
            view,
            sampler,
            width,
            height,
            depth,
        }
    }

    pub async fn to_vec<T: Convert2Vec>(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<T> {

        let size = (self.width * self.height * self.depth * 4) as u64;
        
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size, 
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new(self.width * 4).unwrap()), 
                    rows_per_image: Some(NonZeroU32::new(self.depth).unwrap()),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.depth,
            },
        );
        queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);

        let res: Vec<T>;

        buffer_future.await.expect("failed"); 
        let data = buffer_slice.get_mapped_range();
        res = Convert2Vec::convert(&data);
        res
    }

    /// Create 1d texture array from u8 data. This data is sampled as uninterpolated values from
    /// shader (nearest neighbor).  
    pub fn create_texture_array<T: Pod>(
                queue: &wgpu::Queue,
                device: &wgpu::Device,
                data: &[T], //bytemuck::cast_slice(&t)
                texture_format: wgpu::TextureFormat) -> Self {

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(wgpu::CompareFunction::Equal),
            ..Default::default()
        });

        let width = data.len() as u32;// TODO: create check system (texture_format).
        //let width = data.len() as u32 / 4; // TODO: create check system (texture_format).
        let height: u32 = 1;
        let depth: u32 = 1;

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: depth,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: texture_format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(std::mem::size_of::<T>() as u32 * width).unwrap()),
                rows_per_image: Some(NonZeroU32::new(1).unwrap()),
            },
            texture_extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(texture_format),
            dimension: Some(wgpu::TextureViewDimension::D1),
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let texture_type = TextureType::Diffuse;

        Self {
            texture_type, 
            texture,
            view,
            sampler,
            width,
            height,
            depth,
        }
    }
}
