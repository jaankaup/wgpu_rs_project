use crate::misc::Convert2Vec;
use std::num::NonZeroU32;

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

    pub fn create_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, label: Option<&str>) -> Self {

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
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT, // TODO: SAMPLED?
            //    | wgpu::TextureUsage::SAMPLED
            //    | wgpu::TextureUsage::COPY_SRC,
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
    pub fn create_from_bytes(queue: &wgpu::Queue, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, sample_count : u32, bytes: &[u8], label: Option<&str>) -> Self {

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
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        log::info!("Writing texture.");
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
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
        log::info!("Writing texture, OK.");

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(sc_desc.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            //aspect: wgpu::TextureAspect::default(),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
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

    pub fn create_texture2d(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, sample_count: u32, width: u32, height: u32) -> Self {

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
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(sc_desc.format),// gpu::TextureFormat::Rgba8UnormSrgb,
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
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
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::COPY_SRC,
            label: None,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None, // TODO: add label to function parameter list
            format: Some(*format),// wgpu::TextureFormat::Rgba8UnormSrgb,
            dimension: Some(wgpu::TextureViewDimension::D3),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
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
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
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

    /// Creates the tritable texture for marching cubes.
    /// Creates data in rgba from. 
    pub fn create_tritable(queue: &wgpu::Queue, device: &wgpu::Device) -> Self {
        let data: Vec<u8> = vec![
       255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 , // Case: 0
        0,  8,  3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,    // Case: 1
        0,  1,  9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,    // Case: 2
        1,  8,  3,  9,  8,  1, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  8,  3,  1,  2, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9,  2, 10,  0,  2,  9, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        2,  8,  3,  2, 10,  8, 10,  9,  8, 255, 255, 255, 255, 255, 255 ,
        3, 11,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0, 11,  2,  8, 11,  0, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  9,  0,  2,  3, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1, 11,  2,  1,  9, 11,  9,  8, 11, 255, 255, 255, 255, 255, 255 ,
        3, 10,  1, 11, 10,  3, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0, 10,  1,  0,  8, 10,  8, 11, 10, 255, 255, 255, 255, 255, 255 ,
        3,  9,  0,  3, 11,  9, 11, 10,  9, 255, 255, 255, 255, 255, 255 ,
        9,  8, 10, 10,  8, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  7,  8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  3,  0,  7,  3,  4, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  1,  9,  8,  4,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  1,  9,  4,  7,  1,  7,  3,  1, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10,  8,  4,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  4,  7,  3,  0,  4,  1,  2, 10, 255, 255, 255, 255, 255, 255 ,
        9,  2, 10,  9,  0,  2,  8,  4,  7, 255, 255, 255, 255, 255, 255 ,
        2, 10,  9,  2,  9,  7,  2,  7,  3,  7,  9,  4, 255, 255, 255 ,
        8,  4,  7,  3, 11,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       11,  4,  7, 11,  2,  4,  2,  0,  4, 255, 255, 255, 255, 255, 255 ,
        9,  0,  1,  8,  4,  7,  2,  3, 11, 255, 255, 255, 255, 255, 255 ,
        4,  7, 11,  9,  4, 11,  9, 11,  2,  9,  2,  1, 255, 255, 255 ,
        3, 10,  1,  3, 11, 10,  7,  8,  4, 255, 255, 255, 255, 255, 255 ,
        1, 11, 10,  1,  4, 11,  1,  0,  4,  7, 11,  4, 255, 255, 255 ,
        4,  7,  8,  9,  0, 11,  9, 11, 10, 11,  0,  3, 255, 255, 255 ,
        4,  7, 11,  4, 11,  9,  9, 11, 10, 255, 255, 255, 255, 255, 255 ,
        9,  5,  4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9,  5,  4,  0,  8,  3, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  5,  4,  1,  5,  0, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        8,  5,  4,  8,  3,  5,  3,  1,  5, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10,  9,  5,  4, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  0,  8,  1,  2, 10,  4,  9,  5, 255, 255, 255, 255, 255, 255 ,
        5,  2, 10,  5,  4,  2,  4,  0,  2, 255, 255, 255, 255, 255, 255 ,
        2, 10,  5,  3,  2,  5,  3,  5,  4,  3,  4,  8, 255, 255, 255 ,
        9,  5,  4,  2,  3, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0, 11,  2,  0,  8, 11,  4,  9,  5, 255, 255, 255, 255, 255, 255 ,
        0,  5,  4,  0,  1,  5,  2,  3, 11, 255, 255, 255, 255, 255, 255 ,
        2,  1,  5,  2,  5,  8,  2,  8, 11,  4,  8,  5, 255, 255, 255 ,
       10,  3, 11, 10,  1,  3,  9,  5,  4, 255, 255, 255, 255, 255, 255 ,
        4,  9,  5,  0,  8,  1,  8, 10,  1,  8, 11, 10, 255, 255, 255 ,
        5,  4,  0,  5,  0, 11,  5, 11, 10, 11,  0,  3, 255, 255, 255 ,
        5,  4,  8,  5,  8, 10, 10,  8, 11, 255, 255, 255, 255, 255, 255 ,
        9,  7,  8,  5,  7,  9, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9,  3,  0,  9,  5,  3,  5,  7,  3, 255, 255, 255, 255, 255, 255 ,
        0,  7,  8,  0,  1,  7,  1,  5,  7, 255, 255, 255, 255, 255, 255 ,
        1,  5,  3,  3,  5,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9,  7,  8,  9,  5,  7, 10,  1,  2, 255, 255, 255, 255, 255, 255 ,
       10,  1,  2,  9,  5,  0,  5,  3,  0,  5,  7,  3, 255, 255, 255 ,
        8,  0,  2,  8,  2,  5,  8,  5,  7, 10,  5,  2, 255, 255, 255 ,
        2, 10,  5,  2,  5,  3,  3,  5,  7, 255, 255, 255, 255, 255, 255 ,
        7,  9,  5,  7,  8,  9,  3, 11,  2, 255, 255, 255, 255, 255, 255 ,
        9,  5,  7,  9,  7,  2,  9,  2,  0,  2,  7, 11, 255, 255, 255 ,
        2,  3, 11,  0,  1,  8,  1,  7,  8,  1,  5,  7, 255, 255, 255 ,
       11,  2,  1, 11,  1,  7,  7,  1,  5, 255, 255, 255, 255, 255, 255 ,
        9,  5,  8,  8,  5,  7, 10,  1,  3, 10,  3, 11, 255, 255, 255 ,
        5,  7,  0,  5,  0,  9,  7, 11,  0,  1,  0, 10, 11, 10,  0 ,
       11, 10,  0, 11,  0,  3, 10,  5,  0,  8,  0,  7,  5,  7,  0 ,
       11, 10,  5,  7, 11,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       10,  6,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  8,  3,  5, 10,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9,  0,  1,  5, 10,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  8,  3,  1,  9,  8,  5, 10,  6, 255, 255, 255, 255, 255, 255 ,
        1,  6,  5,  2,  6,  1, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  6,  5,  1,  2,  6,  3,  0,  8, 255, 255, 255, 255, 255, 255 ,
        9,  6,  5,  9,  0,  6,  0,  2,  6, 255, 255, 255, 255, 255, 255 ,
        5,  9,  8,  5,  8,  2,  5,  2,  6,  3,  2,  8, 255, 255, 255 ,
        2,  3, 11, 10,  6,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       11,  0,  8, 11,  2,  0, 10,  6,  5, 255, 255, 255, 255, 255, 255 ,
        0,  1,  9,  2,  3, 11,  5, 10,  6, 255, 255, 255, 255, 255, 255 ,
        5, 10,  6,  1,  9,  2,  9, 11,  2,  9,  8, 11, 255, 255, 255 ,
        6,  3, 11,  6,  5,  3,  5,  1,  3, 255, 255, 255, 255, 255, 255 ,
        0,  8, 11,  0, 11,  5,  0,  5,  1,  5, 11,  6, 255, 255, 255 ,
        3, 11,  6,  0,  3,  6,  0,  6,  5,  0,  5,  9, 255, 255, 255 ,
        6,  5,  9,  6,  9, 11, 11,  9,  8, 255, 255, 255, 255, 255, 255 ,
        5, 10,  6,  4,  7,  8, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  3,  0,  4,  7,  3,  6,  5, 10, 255, 255, 255, 255, 255, 255 ,
        1,  9,  0,  5, 10,  6,  8,  4,  7, 255, 255, 255, 255, 255, 255 ,
       10,  6,  5,  1,  9,  7,  1,  7,  3,  7,  9,  4, 255, 255, 255 ,
        6,  1,  2,  6,  5,  1,  4,  7,  8, 255, 255, 255, 255, 255, 255 ,
        1,  2,  5,  5,  2,  6,  3,  0,  4,  3,  4,  7, 255, 255, 255 ,
        8,  4,  7,  9,  0,  5,  0,  6,  5,  0,  2,  6, 255, 255, 255 ,
        7,  3,  9,  7,  9,  4,  3,  2,  9,  5,  9,  6,  2,  6,  9 ,
        3, 11,  2,  7,  8,  4, 10,  6,  5, 255, 255, 255, 255, 255, 255 ,
        5, 10,  6,  4,  7,  2,  4,  2,  0,  2,  7, 11, 255, 255, 255 ,
        0,  1,  9,  4,  7,  8,  2,  3, 11,  5, 10,  6, 255, 255, 255 ,
        9,  2,  1,  9, 11,  2,  9,  4, 11,  7, 11,  4,  5, 10,  6 ,
        8,  4,  7,  3, 11,  5,  3,  5,  1,  5, 11,  6, 255, 255, 255 ,
        5,  1, 11,  5, 11,  6,  1,  0, 11,  7, 11,  4,  0,  4, 11 ,
        0,  5,  9,  0,  6,  5,  0,  3,  6, 11,  6,  3,  8,  4,  7 ,
        6,  5,  9,  6,  9, 11,  4,  7,  9,  7, 11,  9, 255, 255, 255 ,
       10,  4,  9,  6,  4, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4, 10,  6,  4,  9, 10,  0,  8,  3, 255, 255, 255, 255, 255, 255 ,
       10,  0,  1, 10,  6,  0,  6,  4,  0, 255, 255, 255, 255, 255, 255 ,
        8,  3,  1,  8,  1,  6,  8,  6,  4,  6,  1, 10, 255, 255, 255 ,
        1,  4,  9,  1,  2,  4,  2,  6,  4, 255, 255, 255, 255, 255, 255 ,
        3,  0,  8,  1,  2,  9,  2,  4,  9,  2,  6,  4, 255, 255, 255 ,
        0,  2,  4,  4,  2,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        8,  3,  2,  8,  2,  4,  4,  2,  6, 255, 255, 255, 255, 255, 255 ,
       10,  4,  9, 10,  6,  4, 11,  2,  3, 255, 255, 255, 255, 255, 255 ,
        0,  8,  2,  2,  8, 11,  4,  9, 10,  4, 10,  6, 255, 255, 255 ,
        3, 11,  2,  0,  1,  6,  0,  6,  4,  6,  1, 10, 255, 255, 255 ,
        6,  4,  1,  6,  1, 10,  4,  8,  1,  2,  1, 11,  8, 11,  1 ,
        9,  6,  4,  9,  3,  6,  9,  1,  3, 11,  6,  3, 255, 255, 255 ,
        8, 11,  1,  8,  1,  0, 11,  6,  1,  9,  1,  4,  6,  4,  1 ,
        3, 11,  6,  3,  6,  0,  0,  6,  4, 255, 255, 255, 255, 255, 255 ,
        6,  4,  8, 11,  6,  8, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        7, 10,  6,  7,  8, 10,  8,  9, 10, 255, 255, 255, 255, 255, 255 ,
        0,  7,  3,  0, 10,  7,  0,  9, 10,  6,  7, 10, 255, 255, 255 ,
       10,  6,  7,  1, 10,  7,  1,  7,  8,  1,  8,  0, 255, 255, 255 ,
       10,  6,  7, 10,  7,  1,  1,  7,  3, 255, 255, 255, 255, 255, 255 ,
        1,  2,  6,  1,  6,  8,  1,  8,  9,  8,  6,  7, 255, 255, 255 ,
        2,  6,  9,  2,  9,  1,  6,  7,  9,  0,  9,  3,  7,  3,  9 ,
        7,  8,  0,  7,  0,  6,  6,  0,  2, 255, 255, 255, 255, 255, 255 ,
        7,  3,  2,  6,  7,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        2,  3, 11, 10,  6,  8, 10,  8,  9,  8,  6,  7, 255, 255, 255 ,
        2,  0,  7,  2,  7, 11,  0,  9,  7,  6,  7, 10,  9, 10,  7 ,
        1,  8,  0,  1,  7,  8,  1, 10,  7,  6,  7, 10,  2,  3, 11 ,
       11,  2,  1, 11,  1,  7, 10,  6,  1,  6,  7,  1, 255, 255, 255 ,
        8,  9,  6,  8,  6,  7,  9,  1,  6, 11,  6,  3,  1,  3,  6 ,
        0,  9,  1, 11,  6,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        7,  8,  0,  7,  0,  6,  3, 11,  0, 11,  6,  0, 255, 255, 255 ,
        7, 11,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        7,  6, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  0,  8, 11,  7,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  1,  9, 11,  7,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        8,  1,  9,  8,  3,  1, 11,  7,  6, 255, 255, 255, 255, 255, 255 ,
       10,  1,  2,  6, 11,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10,  3,  0,  8,  6, 11,  7, 255, 255, 255, 255, 255, 255 ,
        2,  9,  0,  2, 10,  9,  6, 11,  7, 255, 255, 255, 255, 255, 255 ,
        6, 11,  7,  2, 10,  3, 10,  8,  3, 10,  9,  8, 255, 255, 255 ,
        7,  2,  3,  6,  2,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        7,  0,  8,  7,  6,  0,  6,  2,  0, 255, 255, 255, 255, 255, 255 ,
        2,  7,  6,  2,  3,  7,  0,  1,  9, 255, 255, 255, 255, 255, 255 ,
        1,  6,  2,  1,  8,  6,  1,  9,  8,  8,  7,  6, 255, 255, 255 ,
       10,  7,  6, 10,  1,  7,  1,  3,  7, 255, 255, 255, 255, 255, 255 ,
       10,  7,  6,  1,  7, 10,  1,  8,  7,  1,  0,  8, 255, 255, 255 ,
        0,  3,  7,  0,  7, 10,  0, 10,  9,  6, 10,  7, 255, 255, 255 ,
        7,  6, 10,  7, 10,  8,  8, 10,  9, 255, 255, 255, 255, 255, 255 ,
        6,  8,  4, 11,  8,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  6, 11,  3,  0,  6,  0,  4,  6, 255, 255, 255, 255, 255, 255 ,
        8,  6, 11,  8,  4,  6,  9,  0,  1, 255, 255, 255, 255, 255, 255 ,
        9,  4,  6,  9,  6,  3,  9,  3,  1, 11,  3,  6, 255, 255, 255 ,
        6,  8,  4,  6, 11,  8,  2, 10,  1, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10,  3,  0, 11,  0,  6, 11,  0,  4,  6, 255, 255, 255 ,
        4, 11,  8,  4,  6, 11,  0,  2,  9,  2, 10,  9, 255, 255, 255 ,
       10,  9,  3, 10,  3,  2,  9,  4,  3, 11,  3,  6,  4,  6,  3 ,
        8,  2,  3,  8,  4,  2,  4,  6,  2, 255, 255, 255, 255, 255, 255 ,
        0,  4,  2,  4,  6,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  9,  0,  2,  3,  4,  2,  4,  6,  4,  3,  8, 255, 255, 255 ,
        1,  9,  4,  1,  4,  2,  2,  4,  6, 255, 255, 255, 255, 255, 255 ,
        8,  1,  3,  8,  6,  1,  8,  4,  6,  6, 10,  1, 255, 255, 255 ,
       10,  1,  0, 10,  0,  6,  6,  0,  4, 255, 255, 255, 255, 255, 255 ,
        4,  6,  3,  4,  3,  8,  6, 10,  3,  0,  3,  9, 10,  9,  3 ,
       10,  9,  4,  6, 10,  4, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  9,  5,  7,  6, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  8,  3,  4,  9,  5, 11,  7,  6, 255, 255, 255, 255, 255, 255 ,
        5,  0,  1,  5,  4,  0,  7,  6, 11, 255, 255, 255, 255, 255, 255 ,
       11,  7,  6,  8,  3,  4,  3,  5,  4,  3,  1,  5, 255, 255, 255 ,
        9,  5,  4, 10,  1,  2,  7,  6, 11, 255, 255, 255, 255, 255, 255 ,
        6, 11,  7,  1,  2, 10,  0,  8,  3,  4,  9,  5, 255, 255, 255 ,
        7,  6, 11,  5,  4, 10,  4,  2, 10,  4,  0,  2, 255, 255, 255 ,
        3,  4,  8,  3,  5,  4,  3,  2,  5, 10,  5,  2, 11,  7,  6 ,
        7,  2,  3,  7,  6,  2,  5,  4,  9, 255, 255, 255, 255, 255, 255 ,
        9,  5,  4,  0,  8,  6,  0,  6,  2,  6,  8,  7, 255, 255, 255 ,
        3,  6,  2,  3,  7,  6,  1,  5,  0,  5,  4,  0, 255, 255, 255 ,
        6,  2,  8,  6,  8,  7,  2,  1,  8,  4,  8,  5,  1,  5,  8 ,
        9,  5,  4, 10,  1,  6,  1,  7,  6,  1,  3,  7, 255, 255, 255 ,
        1,  6, 10,  1,  7,  6,  1,  0,  7,  8,  7,  0,  9,  5,  4 ,
        4,  0, 10,  4, 10,  5,  0,  3, 10,  6, 10,  7,  3,  7, 10 ,
        7,  6, 10,  7, 10,  8,  5,  4, 10,  4,  8, 10, 255, 255, 255 ,
        6,  9,  5,  6, 11,  9, 11,  8,  9, 255, 255, 255, 255, 255, 255 ,
        3,  6, 11,  0,  6,  3,  0,  5,  6,  0,  9,  5, 255, 255, 255 ,
        0, 11,  8,  0,  5, 11,  0,  1,  5,  5,  6, 11, 255, 255, 255 ,
        6, 11,  3,  6,  3,  5,  5,  3,  1, 255, 255, 255, 255, 255, 255 ,
        1,  2, 10,  9,  5, 11,  9, 11,  8, 11,  5,  6, 255, 255, 255 ,
        0, 11,  3,  0,  6, 11,  0,  9,  6,  5,  6,  9,  1,  2, 10 ,
       11,  8,  5, 11,  5,  6,  8,  0,  5, 10,  5,  2,  0,  2,  5 ,
        6, 11,  3,  6,  3,  5,  2, 10,  3, 10,  5,  3, 255, 255, 255 ,
        5,  8,  9,  5,  2,  8,  5,  6,  2,  3,  8,  2, 255, 255, 255 ,
        9,  5,  6,  9,  6,  0,  0,  6,  2, 255, 255, 255, 255, 255, 255 ,
        1,  5,  8,  1,  8,  0,  5,  6,  8,  3,  8,  2,  6,  2,  8 ,
        1,  5,  6,  2,  1,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  3,  6,  1,  6, 10,  3,  8,  6,  5,  6,  9,  8,  9,  6 ,
       10,  1,  0, 10,  0,  6,  9,  5,  0,  5,  6,  0, 255, 255, 255 ,
        0,  3,  8,  5,  6, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       10,  5,  6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       11,  5, 10,  7,  5, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
       11,  5, 10, 11,  7,  5,  8,  3,  0, 255, 255, 255, 255, 255, 255 ,
        5, 11,  7,  5, 10, 11,  1,  9,  0, 255, 255, 255, 255, 255, 255 ,
       10,  7,  5, 10, 11,  7,  9,  8,  1,  8,  3,  1, 255, 255, 255 ,
       11,  1,  2, 11,  7,  1,  7,  5,  1, 255, 255, 255, 255, 255, 255 ,
        0,  8,  3,  1,  2,  7,  1,  7,  5,  7,  2, 11, 255, 255, 255 ,
        9,  7,  5,  9,  2,  7,  9,  0,  2,  2, 11,  7, 255, 255, 255 ,
        7,  5,  2,  7,  2, 11,  5,  9,  2,  3,  2,  8,  9,  8,  2 ,
        2,  5, 10,  2,  3,  5,  3,  7,  5, 255, 255, 255, 255, 255, 255 ,
        8,  2,  0,  8,  5,  2,  8,  7,  5, 10,  2,  5, 255, 255, 255 ,
        9,  0,  1,  5, 10,  3,  5,  3,  7,  3, 10,  2, 255, 255, 255 ,
        9,  8,  2,  9,  2,  1,  8,  7,  2, 10,  2,  5,  7,  5,  2 ,
        1,  3,  5,  3,  7,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  8,  7,  0,  7,  1,  1,  7,  5, 255, 255, 255, 255, 255, 255 ,
        9,  0,  3,  9,  3,  5,  5,  3,  7, 255, 255, 255, 255, 255, 255 ,
        9,  8,  7,  5,  9,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        5,  8,  4,  5, 10,  8, 10, 11,  8, 255, 255, 255, 255, 255, 255 ,
        5,  0,  4,  5, 11,  0,  5, 10, 11, 11,  3,  0, 255, 255, 255 ,
        0,  1,  9,  8,  4, 10,  8, 10, 11, 10,  4,  5, 255, 255, 255 ,
       10, 11,  4, 10,  4,  5, 11,  3,  4,  9,  4,  1,  3,  1,  4 ,
        2,  5,  1,  2,  8,  5,  2, 11,  8,  4,  5,  8, 255, 255, 255 ,
        0,  4, 11,  0, 11,  3,  4,  5, 11,  2, 11,  1,  5,  1, 11 ,
        0,  2,  5,  0,  5,  9,  2, 11,  5,  4,  5,  8, 11,  8,  5 ,
        9,  4,  5,  2, 11,  3, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        2,  5, 10,  3,  5,  2,  3,  4,  5,  3,  8,  4, 255, 255, 255 ,
        5, 10,  2,  5,  2,  4,  4,  2,  0, 255, 255, 255, 255, 255, 255 ,
        3, 10,  2,  3,  5, 10,  3,  8,  5,  4,  5,  8,  0,  1,  9 ,
        5, 10,  2,  5,  2,  4,  1,  9,  2,  9,  4,  2, 255, 255, 255 ,
        8,  4,  5,  8,  5,  3,  3,  5,  1, 255, 255, 255, 255, 255, 255 ,
        0,  4,  5,  1,  0,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        8,  4,  5,  8,  5,  3,  9,  0,  5,  0,  3,  5, 255, 255, 255 ,
        9,  4,  5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4, 11,  7,  4,  9, 11,  9, 10, 11, 255, 255, 255, 255, 255, 255 ,
        0,  8,  3,  4,  9,  7,  9, 11,  7,  9, 10, 11, 255, 255, 255 ,
        1, 10, 11,  1, 11,  4,  1,  4,  0,  7,  4, 11, 255, 255, 255 ,
        3,  1,  4,  3,  4,  8,  1, 10,  4,  7,  4, 11, 10, 11,  4 ,
        4, 11,  7,  9, 11,  4,  9,  2, 11,  9,  1,  2, 255, 255, 255 ,
        9,  7,  4,  9, 11,  7,  9,  1, 11,  2, 11,  1,  0,  8,  3 ,
       11,  7,  4, 11,  4,  2,  2,  4,  0, 255, 255, 255, 255, 255, 255 ,
       11,  7,  4, 11,  4,  2,  8,  3,  4,  3,  2,  4, 255, 255, 255 ,
        2,  9, 10,  2,  7,  9,  2,  3,  7,  7,  4,  9, 255, 255, 255 ,
        9, 10,  7,  9,  7,  4, 10,  2,  7,  8,  7,  0,  2,  0,  7 ,
        3,  7, 10,  3, 10,  2,  7,  4, 10,  1, 10,  0,  4,  0, 10 ,
        1, 10,  2,  8,  7,  4, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  9,  1,  4,  1,  7,  7,  1,  3, 255, 255, 255, 255, 255, 255 ,
        4,  9,  1,  4,  1,  7,  0,  8,  1,  8,  7,  1, 255, 255, 255 ,
        4,  0,  3,  7,  4,  3, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        4,  8,  7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        9, 10,  8, 10, 11,  8, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  0,  9,  3,  9, 11, 11,  9, 10, 255, 255, 255, 255, 255, 255 ,
        0,  1, 10,  0, 10,  8,  8, 10, 11, 255, 255, 255, 255, 255, 255 ,
        3,  1, 10, 11,  3, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  2, 11,  1, 11,  9,  9, 11,  8, 255, 255, 255, 255, 255, 255 ,
        3,  0,  9,  3,  9, 11,  1,  2,  9,  2, 11,  9, 255, 255, 255 ,
        0,  2, 11,  8,  0, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        3,  2, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        2,  3,  8,  2,  8, 10, 10,  8,  9, 255, 255, 255, 255, 255, 255 ,
        9, 10,  2,  0,  9,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        2,  3,  8,  2,  8, 10,  0,  1,  8,  1, 10,  8, 255, 255, 255 ,
        1, 10,  2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        1,  3,  8,  9,  1,  8, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  9,  1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 ,
        0,  3,  8, 255, 255, 255, 255 ,255 ,255, 255 ,255 ,255, 255 ,255 ,255 ,
       255, 255, 255, 255, 255, 255, 255 ,255 ,255, 255 ,255 ,255, 255 ,255 ,255];

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            //compare: Some(wgpu::CompareFunction::Never),
            compare: Some(wgpu::CompareFunction::Equal),
            ..Default::default()
        });

        // Add alpha component to the data. 
        
        //let mut buffer = vec![0; 5120];
        let mut buffer = Vec::new(); //vec![0; 5120];

        // Add the aplha value (== 255) for each pixel.
        let mut counter = 0; 
        for i in 0..data.len() {
            if counter == 2 { counter = 0; buffer.push(data[i]); buffer.push(255); }
            else {
                buffer.push(data[i]);
                counter = counter + 1;
            }
        }

        let width = 1280;
        let height = 1;
        let depth = 1;

        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: depth,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            // array_layer_count: 1, // only one texture now
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D1,
            //format: wgpu::TextureFormat::Rgba8Uint,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // TODO: sc_dest.format
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(5120).unwrap()),
                rows_per_image: Some(NonZeroU32::new(1).unwrap()),
            },
            texture_extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),// TODO: sc_dest.format
            dimension: Some(wgpu::TextureViewDimension::D1),
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1), //Some(0),
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
