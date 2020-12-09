#[cfg(target_arch = "wasm32")]
use futures::task::LocalSpawn;

use winit::{
    event::{Event, WindowEvent,KeyboardInput,ElementState,VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    //window::Window
};

use crate::input::InputCache;

/// A trait for wgpu-rs based application.
pub trait Application: Sized + 'static {

    /// Creates the Application.
    fn init(configuration: &WGPUConfiguration) -> Self;

    /// The render function for application.
    fn render(self);

    /// A function that handles inputs.
    fn input(self);

    /// A function for resizing.
    fn resize(self);

    /// A function for updating the state of the application.
    fn update(self);
}

/// A trait for Loops.
pub trait Loop: Sized + 'static {

    /// Initialize loop.
    fn init() -> Self;

    /// Run function that starts the loop. Beware: run takes ownership of application and
    /// configuration.
    fn run<A: Application>(self, application: A, configuration: WGPUConfiguration);
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

    fn run<A: Application>(self, mut application: A, WGPUConfiguration {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
        swap_chain,
        sc_desc
        }: WGPUConfiguration,) {

    // Create thread pool and spawner for native version.
    #[cfg(not(target_arch = "wasm32"))]
    let (mut pool, _spawner) = {

        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };

    // Define spawner for wasm version.
    #[cfg(target_arch = "wasm32")]
    let spawner = {
        use futures::{future::LocalFutureObj, task::SpawnError};
        use winit::platform::web::WindowExtWebSys;

        struct WebSpawner {}
        impl LocalSpawn for WebSpawner {
            fn spawn_local_obj(
                &self,
                future: LocalFutureObj<'static, ()>,
            ) -> Result<(), SpawnError> {
                Ok(wasm_bindgen_futures::spawn_local(future))
            }
        }

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        WebSpawner {}
    };

    let mut input = InputCache::init();

    // Launch the loop.
    event_loop.run(move |event, _, control_flow| {

        // Force the ownerships to this closure.
        let _ = (&window,
                &instance,
                &size,
                &surface,
                &adapter,
                &device,
                &queue,
                &swap_chain,
                &sc_desc,
                &mut application,
                &mut input);

        *control_flow = ControlFlow::Poll;

        match event {

            // Events except RedrawRequested are reported.
            Event::MainEventsCleared => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    pool.run_until_stalled();
                }

                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
            }
            Event::WindowEvent { event, ..} => {
                // Update input cache.
                input.pre_update();
                input.update(&event);

                match event {
                    WindowEvent::Resized(size) => {
                        // TODO: change the size and and modify the sc_desc and create new swap_chain.
                        //application.resize(size);
                    }
                    WindowEvent::CloseRequested => {
                        // TODO: application.close()
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                //state.render(&window);
            }
            _ => { } // Any other events
        } // match event
    }); // run

    }
}


/// Initializes wgpu-rs system. TODO: finish the Result<...>.
pub async fn setup<P: WGPUFeatures>(title: &str) -> Result<WGPUConfiguration, &'static str> {

    println!("YEEEAAAAH");

    let title = title.to_owned();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACE");
        subscriber::initialize_default_subscriber(
            chrome_tracing_dir.as_ref().map(std::path::Path::new).ok(),
        );
    };

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
        console_log::init().expect("could not initialize logger");
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

    let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::BackendBit::VULKAN,
            "metal" => wgpu::BackendBit::METAL,
            "dx12" => wgpu::BackendBit::DX12,
            "dx11" => wgpu::BackendBit::DX11,
            "gl" => wgpu::BackendBit::GL,
            "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
            other => panic!("Unknown backend: {}", other),
        }
    } else {
        wgpu::BackendBit::PRIMARY
    };
    let instance = wgpu::Instance::new(backend);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();
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
                shader_validation: true,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        // TODO: Allow srgb unconditionally
        format: if cfg!(target_arch = "wasm32") {
            wgpu::TextureFormat::Bgra8Unorm
        } else {
            wgpu::TextureFormat::Bgra8UnormSrgb
        },
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
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
    let configuration = futures::executor::block_on(setup::<F>("jihuu")).expect("Failed to create WGPUConfiguration.");
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
        //basic_loop<HelloApp>(application: A, WGPUConfiguration {
        lo.run(app, configuration); 
    });
}
