use crate::buffer::{to_vec, buffer_from_data};
use crate::wgpu_system::*;
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
    pub fn init(device: &wgpu::Device, initial_values: &Vec<u32>) -> Self {

        assert!(initial_values.len() > 0, format!("{} > 0", initial_values.len()));

        //let data = vec!(initial_value ; capacity as usize);
        //println!("{:?}", initial_values);

        let histogram = buffer_from_data::<u32>(
            &device,
            &initial_values,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE,
            None);

        Self {
            histogram: histogram,
            data: initial_values.to_vec(),
        }
    }

    /// Get 
    pub fn get_values(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u32> {

        let result = to_vec::<u32>(&device,
                                   &queue,
                                   &self.histogram,
                                   0 as wgpu::BufferAddress,
                                   (std::mem::size_of::<u32>() * self.data.len()) as wgpu::BufferAddress);
        //#[cfg(not(target_arch = "wasm32"))] {
        //let result = pollster::block_on(
        //                to_vec::<u32>(&device,
        //                              &queue,
        //                              &self.histogram,
        //                              0 as wgpu::BufferAddress,
        //                              (std::mem::size_of::<u32>() * self.data.len()) as wgpu::BufferAddress));
        //    result
        //}
        //#[cfg(target_arch = "wasm32")] {
        //    let spawner = async_executor::LocalExecutor::new();
        //    let result: Vec<u32> = spawner.run(
        //                    to_vec::<u32>(&device,
        //                                  &queue,
        //                                  &self.histogram,
        //                                  0 as wgpu::BufferAddress,
        //                                  (std::mem::size_of::<u32>() * self.data.len()) as wgpu::BufferAddress)
        //    );
        //    result
        //}
        result
    }
    
    pub fn get_histogram(&self) -> &wgpu::Buffer {
        &self.histogram
    }

    pub fn set_values_cpu_version(&self, queue: &wgpu::Queue, value: &Vec<u32>)
    {
        // Make sure the updated values are the same size as old values.
        assert!(value.len() == self.data.len(), format!("{} > {}", self.data.len(), value.len()));

        queue.write_buffer(
            &self.histogram,
            0,
            bytemuck::cast_slice(&value)
        );
    }

    pub fn reset_cpu_version(&self, queue: &wgpu::Queue, value: u32) {
        queue.write_buffer(
            &self.histogram,
            0,
            bytemuck::cast_slice(&vec![value ; self.data.len() as usize])
        );
    }
}
