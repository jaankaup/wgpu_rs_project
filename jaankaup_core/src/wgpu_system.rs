//#[cfg(target_arch = "wasm32")]
//use futures::task::LocalSpawn;
use std::future::Future;
//use env_logger::*;

#[cfg(not(target_arch = "wasm32"))]
use simple_logger::SimpleLogger;

use log::LevelFilter;
//use simplelog::*;
//use std::fs::File;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    //window::Window
};

use crate::input::InputCache;

/// A trait for wgpu-rs based application.
pub trait Application: Sized + 'static {

    /// Creates the Application.
    fn init(configuration: &WGPUConfiguration) -> Self;

    /// The render function for application.
    fn render(&mut self,
              device: &wgpu::Device,
              queue: &mut wgpu::Queue,
              swap_chain: &mut wgpu::SwapChain,
              surface: &wgpu::Surface,
              sc_desc: &wgpu::SwapChainDescriptor);

    /// A function that handles inputs.
    fn input(&mut self, input_cache: &InputCache);

    /// A function for resizing.
    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, new_size: winit::dpi::PhysicalSize<u32>);

    /// A function for updating the state of the application.
    fn update(&self);
}

/// A trait for Loops.
pub trait Loop: Sized + 'static {

    /// Initialize loop.
    fn init() -> Self;

    /// Run function that starts the loop. Beware: run takes ownership of application and
    /// configuration.
    fn run<A: Application>(&self, application: A, configuration: WGPUConfiguration);
}

/// A struct that holds the wgpu-rs application resources.
pub struct WGPUConfiguration {
    pub window: winit::window::Window,
    pub event_loop: EventLoop<()>,
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SwapChainDescriptor,
}

/// A trait to configure wgpu-rs engine. TODO: Do we need this? 'static + Sized
pub trait WGPUFeatures: Sized + 'static {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
}

/// A basic loop.
pub struct BasicLoop { }

impl Loop for BasicLoop {

    fn init() -> Self {
        BasicLoop {}
    }

    fn run<A: Application>(&self, mut application: A, WGPUConfiguration {
        window,
        event_loop,
        instance,
        mut size,
        surface,
        adapter,
        device,
        mut queue,
        mut swap_chain,
        mut sc_desc
        }: WGPUConfiguration,) {

    let spawner = Spawner::new();

    let mut input = InputCache::init();

    // Launch the loop.
    event_loop.run(move |event, _, control_flow| {

        // Force the ownerships to this closure.
        let _ = (&window,
                &instance,
                &mut size,
                &surface,
                &adapter,
                &device,
                &mut queue,
                &mut swap_chain,
                &mut sc_desc,
                &mut application,
                &mut input);

        *control_flow = ControlFlow::Poll;

        match event {

            // Events except RedrawRequested are reported.
            Event::MainEventsCleared => {
                // log::info!("MainEventsCleared....");
                input.pre_update();
                #[cfg(not(target_arch = "wasm32"))]
                {
                    //pool.run_until_stalled();
                    spawner.run_until_stalled();
                }
                window.request_redraw();

                //#[cfg(target_arch = "wasm32")]
            }
            Event::WindowEvent { event, ..} => {
                // Update input cache.
                input.update(&event);

                match event {
                    WindowEvent::Resized(new_size) => {
                        // TODO: change the size and and modify the sc_desc and create new swap_chain.
                        size = new_size;
                        sc_desc.width = new_size.width;
                        sc_desc.height = new_size.height;
                        swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        application.resize(&device, &sc_desc, size);
                    }
                    WindowEvent::CloseRequested => {
                        // TODO: application.close()
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }
                
                application.input(&input);
            }
            Event::RedrawRequested(_) => {
                //log::info!("Nyt piirrellaan.");
                application.render(&device, &mut queue, &mut swap_chain, &surface, &sc_desc);
            }
            _ => { } // Any other events
        } // match event
    }); // run

    }
}


/// Initializes wgpu-rs system. TODO: finish the Result<...>.
pub async fn setup<P: WGPUFeatures>(title: &str) -> Result<WGPUConfiguration, &'static str> {

    let title = title.to_owned();
    // env_logger::init();

    #[cfg(not(target_arch = "wasm32"))]
    {
        SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("jaankaup", LevelFilter::Info)
        .with_module_level("hello_project", LevelFilter::Info)
        .init()
        .unwrap();
    }
    // {
    //     let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACE");
    //     subscriber::initialize_default_subscriber(
    //         chrome_tracing_dir.as_ref().map(std::path::Path::new).ok(),
    //     );
    // };

    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);
    #[cfg(windows_OFF)] // TODO
    {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder = builder.with_no_redirection_bitmap(true);
    }
    let window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        //console_log::init().expect("could not initialize logger");
        console_log::init_with_level(log::Level::Trace).expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body.");
    }

    log::info!("Initializing the surface...");
    if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        log::info!("Backend == {}", backend.to_lowercase().as_str());
    }
    let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::BackendBit::VULKAN,
            "metal" => wgpu::BackendBit::METAL,
            "dx12" => wgpu::BackendBit::DX12,
            "dx11" => wgpu::BackendBit::DX11,
            "gl" =>  wgpu::BackendBit::GL,
            "webgpu" =>  wgpu::BackendBit::BROWSER_WEBGPU, 
            other => panic!("Unknown backend: {}", other),
        }
    } else {
        wgpu::BackendBit::PRIMARY
    };
    
    log::info!("Backend == {}", match backend {
            wgpu::BackendBit::VULKAN => "vulkan",
            wgpu::BackendBit::METAL => "metal",
            wgpu::BackendBit::DX12 => "dx12",
            wgpu::BackendBit::DX11 => "dx11",
            wgpu::BackendBit::GL => "gl",
            wgpu::BackendBit::BROWSER_WEBGPU => "webgpu", 
            other => "other",
    });
    let power_preference = if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
        match power_preference.to_lowercase().as_str() {
            "low" => wgpu::PowerPreference::LowPower,
            "high" => wgpu::PowerPreference::HighPerformance,
            other => panic!("Unknown power preference: {}", other),
        }
    } else {
        wgpu::PowerPreference::default()
    };
    let instance = wgpu::Instance::new(backend);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("No suitable GPU adapters found on the system!");
    let optional_features = P::optional_features();
    let required_features = P::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );

    let needed_limits = P::required_limits();

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
                //shader_validation: true,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    log::info!("Color format == {}", match sc_desc.format {
    wgpu::TextureFormat::R8Unorm               => "R8Unorm               ",
    wgpu::TextureFormat::R8Snorm               => "R8Snorm               ",
    wgpu::TextureFormat::R8Uint                => "R8Uint                ",
    wgpu::TextureFormat::R8Sint                => "R8Sint                ",
    wgpu::TextureFormat::R16Uint               => "R16Uint               ",
    wgpu::TextureFormat::R16Sint               => "R16Sint               ",
    wgpu::TextureFormat::R16Float              => "R16Float              ",
    wgpu::TextureFormat::Rg8Unorm              => "Rg8Unorm              ",
    wgpu::TextureFormat::Rg8Snorm              => "Rg8Snorm              ",
    wgpu::TextureFormat::Rg8Uint               => "Rg8Uint               ",
    wgpu::TextureFormat::Rg8Sint               => "Rg8Sint               ",
    wgpu::TextureFormat::R32Uint               => "R32Uint               ",
    wgpu::TextureFormat::R32Sint               => "R32Sint               ",
    wgpu::TextureFormat::R32Float              => "R32Float              ",
    wgpu::TextureFormat::Rg16Uint              => "Rg16Uint              ",
    wgpu::TextureFormat::Rg16Sint              => "Rg16Sint              ",
    wgpu::TextureFormat::Rg16Float             => "Rg16Float             ",
    wgpu::TextureFormat::Rgba8Unorm            => "Rgba8Unorm            ",
    wgpu::TextureFormat::Rgba8UnormSrgb        => "Rgba8UnormSrgb        ",
    wgpu::TextureFormat::Rgba8Snorm            => "Rgba8Snorm            ",
    wgpu::TextureFormat::Rgba8Uint             => "Rgba8Uint             ",
    wgpu::TextureFormat::Rgba8Sint             => "Rgba8Sint             ",
    wgpu::TextureFormat::Bgra8Unorm            => "Bgra8Unorm            ",
    wgpu::TextureFormat::Bgra8UnormSrgb        => "Bgra8UnormSrgb        ",
    wgpu::TextureFormat::Rgb10a2Unorm          => "Rgb10a2Unorm          ",
    wgpu::TextureFormat::Rg11b10Float          => "Rg11b10Float          ",
    wgpu::TextureFormat::Rg32Uint              => "Rg32Uint              ",
    wgpu::TextureFormat::Rg32Sint              => "Rg32Sint              ",
    wgpu::TextureFormat::Rg32Float             => "Rg32Float             ",
    wgpu::TextureFormat::Rgba16Uint            => "Rgba16Uint            ",
    wgpu::TextureFormat::Rgba16Sint            => "Rgba16Sint            ",
    wgpu::TextureFormat::Rgba16Float           => "Rgba16Float           ",
    wgpu::TextureFormat::Rgba32Uint            => "Rgba32Uint            ",
    wgpu::TextureFormat::Rgba32Sint            => "Rgba32Sint            ",
    wgpu::TextureFormat::Rgba32Float           => "Rgba32Float           ",
    wgpu::TextureFormat::Depth32Float          => "Depth32Float          ",
    wgpu::TextureFormat::Depth24Plus           => "Depth24Plus           ",
    wgpu::TextureFormat::Depth24PlusStencil8   => "Depth24PlusStencil8   ",
    wgpu::TextureFormat::Bc1RgbaUnorm          => "Bc1RgbaUnorm          ",
    wgpu::TextureFormat::Bc1RgbaUnormSrgb      => "Bc1RgbaUnormSrgb      ",
    wgpu::TextureFormat::Bc2RgbaUnorm          => "Bc2RgbaUnorm          ",
    wgpu::TextureFormat::Bc2RgbaUnormSrgb      => "Bc2RgbaUnormSrgb      ",
    wgpu::TextureFormat::Bc3RgbaUnorm          => "Bc3RgbaUnorm          ",
    wgpu::TextureFormat::Bc3RgbaUnormSrgb      => "Bc3RgbaUnormSrgb      ",
    //wgpu::TextureFormat::bc4runorm             => "bc4runorm             ",
    wgpu::TextureFormat::Bc4RSnorm             => "Bc4RSnorm             ",
    wgpu::TextureFormat::Bc5RgUnorm            => "Bc5RgUnorm            ",
    wgpu::TextureFormat::Bc5RgSnorm            => "Bc5RgSnorm            ",
    wgpu::TextureFormat::Bc6hRgbUfloat         => "Bc6hRgbUfloat         ",
    wgpu::TextureFormat::Bc6hRgbSfloat         => "Bc6hRgbSfloat         ",
    wgpu::TextureFormat::Bc7RgbaUnorm          => "Bc7RgbaUnorm          ",
    wgpu::TextureFormat::Bc7RgbaUnormSrgb      => "Bc7RgbaUnormSrgb      ",
    wgpu::TextureFormat::Etc2RgbUnorm          => "Etc2RgbUnorm          ",
    wgpu::TextureFormat::Etc2RgbUnormSrgb      => "Etc2RgbUnormSrgb      ",
    wgpu::TextureFormat::Etc2RgbA1Unorm        => "Etc2RgbA1Unorm        ",
    wgpu::TextureFormat::Etc2RgbA1UnormSrgb    => "Etc2RgbA1UnormSrgb    ",
    wgpu::TextureFormat::Etc2RgbA8Unorm        => "Etc2RgbA8Unorm        ",
    wgpu::TextureFormat::Etc2RgbA8UnormSrgb    => "Etc2RgbA8UnormSrgb    ",
    wgpu::TextureFormat::EacRUnorm             => "EacRUnorm             ",
    wgpu::TextureFormat::EacRSnorm             => "EacRSnorm             ",
    wgpu::TextureFormat::EtcRgUnorm            => "EtcRgUnorm            ",
    wgpu::TextureFormat::EtcRgSnorm            => "EtcRgSnorm            ",
    wgpu::TextureFormat::Astc4x4RgbaUnorm      => "Astc4x4RgbaUnorm      ",
    wgpu::TextureFormat::Astc4x4RgbaUnormSrgb  => "Astc4x4RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc5x4RgbaUnorm      => "Astc5x4RgbaUnorm      ",
    wgpu::TextureFormat::Astc5x4RgbaUnormSrgb  => "Astc5x4RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc5x5RgbaUnorm      => "Astc5x5RgbaUnorm      ",
    wgpu::TextureFormat::Astc5x5RgbaUnormSrgb  => "Astc5x5RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc6x5RgbaUnorm      => "Astc6x5RgbaUnorm      ",
    wgpu::TextureFormat::Astc6x5RgbaUnormSrgb  => "Astc6x5RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc6x6RgbaUnorm      => "Astc6x6RgbaUnorm      ",
    wgpu::TextureFormat::Astc6x6RgbaUnormSrgb  => "Astc6x6RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc8x5RgbaUnorm      => "Astc8x5RgbaUnorm      ",
    wgpu::TextureFormat::Astc8x5RgbaUnormSrgb  => "Astc8x5RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc8x6RgbaUnorm      => "Astc8x6RgbaUnorm      ",
    wgpu::TextureFormat::Astc8x6RgbaUnormSrgb  => "Astc8x6RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc10x5RgbaUnorm     => "Astc10x5RgbaUnorm     ",
    wgpu::TextureFormat::Astc10x5RgbaUnormSrgb => "Astc10x5RgbaUnormSrgb ",
    wgpu::TextureFormat::Astc10x6RgbaUnorm     => "Astc10x6RgbaUnorm     ",
    wgpu::TextureFormat::Astc10x6RgbaUnormSrgb => "Astc10x6RgbaUnormSrgb ",
    wgpu::TextureFormat::Astc8x8RgbaUnorm      => "Astc8x8RgbaUnorm      ",
    wgpu::TextureFormat::Astc8x8RgbaUnormSrgb  => "Astc8x8RgbaUnormSrgb  ",
    wgpu::TextureFormat::Astc10x8RgbaUnorm     => "Astc10x8RgbaUnorm     ",
    wgpu::TextureFormat::Astc10x8RgbaUnormSrgb => "Astc10x8RgbaUnormSrgb ",
    wgpu::TextureFormat::Astc10x10RgbaUnorm    => "Astc10x10RgbaUnorm    ",
    wgpu::TextureFormat::Astc10x10RgbaUnormSrgb=> "Astc10x10RgbaUnormSrgb",
    wgpu::TextureFormat::Astc12x10RgbaUnorm    => "Astc12x10RgbaUnorm    ",
    wgpu::TextureFormat::Astc12x10RgbaUnormSrgb=> "Astc12x10RgbaUnormSrgb",
    wgpu::TextureFormat::Astc12x12RgbaUnorm    => "Astc12x12RgbaUnorm    ",
    wgpu::TextureFormat::Astc12x12RgbaUnormSrgb => "Astc12x12RgbaUnormSrgb",
        _ => "Something else",
    });
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    Ok(WGPUConfiguration {
            window: window,
            event_loop: event_loop,
            instance: instance,
            size: size,
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            swap_chain: swap_chain,
            sc_desc: sc_desc,
    })
}

/// Initializes wgpu-rs basic components, application and starts the loop. Native version.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_loop<A: Application, L: Loop, F: WGPUFeatures>() {
    let configuration = pollster::block_on(setup::<F>("jihuu")).expect("Failed to create WGPUConfiguration.");
    let app = A::init(&configuration);
    let lo = L::init();
    lo.run(app, configuration); 
}

/// Initializes wgpu-rs basic components, application and starts the loop. wasm version.
#[cfg(target_arch = "wasm32")]
pub fn run_loop<A: Application, L: Loop, F: WGPUFeatures>() {
    wasm_bindgen_futures::spawn_local(async move {
        let configuration = setup::<F>("jihuu").await.unwrap();
        let app = A::init(&configuration); 
        let lo = L::init();
        lo.run(app, configuration);
    });
}


#[cfg(not(target_arch = "wasm32"))]
pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Spawner {}

#[cfg(target_arch = "wasm32")]
impl Spawner {
    fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
