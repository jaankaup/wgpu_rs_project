pub trait LayoutEntry {
    fn create_bind_group_layouts(device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout>;
}

//pub fn create_bind_group<T>
