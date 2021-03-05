use crate::buffer::{to_vec, buffer_from_data};
/// Information about the invocation counts and data dimensions sizes.
/// TODO: add items_per_thread?
pub struct CompDimensions {
    invocations: wgpu::Buffer, // number of work groups.
    dimensions:  wgpu::Buffer, // the total dimension sizes of the data.
}

impl CompDimensions {
   pub fn init(device: &wgpu::Device, invocations: [u32; 3], dimensions: [u32; 3]) -> Self { 
        Self {
            invocations: buffer_from_data::<u32>(
                            &device,
                            &invocations, //&vec![64,6,64],
                            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE,
                            None),
            dimensions: buffer_from_data::<u32>(
                            &device,
                            &dimensions, //&vec![256,24,256],
                            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE,
                            None)
        }
    }
    pub fn get_invocations(&self) -> &wgpu::Buffer {
        &self.invocations
    }
    pub fn get_dimensions(&self) -> &wgpu::Buffer {
        &self.dimensions
    }
}

/// Histogram struct for GPU purposes. 
pub struct Histogram {
    histogram: wgpu::Buffer,
    data: Vec<u32>,
}

impl Histogram {

    /// Create histogram with given capacity and default value.
    pub fn init(device: &wgpu::Device, capacity: u32, initial_value: u32) -> Self {

        assert!(capacity > 0, format!("{} > 0", capacity));

        let data = vec!(initial_value ; capacity as usize);
        println!("{:?}", data);

        let histogram = buffer_from_data::<u32>(
            &device,
            &data,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE,
            None);

        Self {
            histogram,
            data,
        }
    }

    /// Get 
    pub fn get_values(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u32> {

        #[cfg(not(target_arch = "wasm32"))]
        let result = pollster::block_on(
                        to_vec::<u32>(&device,
                                      &queue,
                                      &self.histogram,
                                      0 as wgpu::BufferAddress,
                                      (std::mem::size_of::<u32>() * self.data.len()) as wgpu::BufferAddress));
        result
    }
    
    pub fn get_histogram(&self) -> &wgpu::Buffer {
        &self.histogram
    }

    pub fn reset_cpu_version(&self, queue: &wgpu::Queue, value: u32) {
        queue.write_buffer(
            &self.histogram,
            0,
            bytemuck::cast_slice(&vec![value ; self.data.len() as usize])
        );
    }
}
