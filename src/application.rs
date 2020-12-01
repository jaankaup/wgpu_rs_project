mod wgpu_system;

pub trait Application: Sized + 'static {
    fn init(self) -> Self;
    fn render(self);
    fn input(self);
    fn resize(self);
    fn update(self);
}

pub trait Loop {
    fn run<A: Application>(configuration: wgpu_system::WGPUConfiguration);
}
