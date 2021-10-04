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
pub use winit::event::VirtualKeyCode as Key;

use crate::input::InputCache;

/// A trait for wgpu-rs based application.
pub trait Application: Sized + 'static {

    /// Creates the Application.
    fn init(configuration: &WGPUConfiguration) -> Self;

    /// The render function for application.
    fn render(&mut self,
              device: &wgpu::Device,
              queue: &mut wgpu::Queue,
              //swap_chain: &mut wgpu::SwapChain,
              surface: &wgpu::Surface,
              sc_desc: &wgpu::SurfaceConfiguration);

    /// A function that handles inputs.
    fn input(&mut self, queue: &wgpu::Queue, input_cache: &InputCache);

    /// A function for resizing.
    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SurfaceConfiguration, new_size: winit::dpi::PhysicalSize<u32>);

    /// A function for updating the state of the application.
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, input: &InputCache);
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
    //pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SurfaceConfiguration,
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
        let mut limits = wgpu::Limits::default();
        //limits.max_storage_buffers_per_pipeline_layout = 8;
        limits.max_storage_buffers_per_shader_stage = 8;
        println!("limits.max_storage_buffer_binding_size == {}", limits.max_storage_buffer_binding_size);
        limits
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
        //mut swap_chain,
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
                // &mut swap_chain,
                &mut sc_desc,
                &mut application,
                &mut input,
                &spawner);

        *control_flow = ControlFlow::Poll;
        //*control_flow = ControlFlow::Wait;

        match event {

            // Event::NewEvents(start_cause) => {
            //     match start_cause {
            //         Init => {}
            //         _ => {}
            //     }
            // }

            Event::LoopDestroyed => {
                // TODO: call clean up code. 
            }

            //Event::MainEventsCleared => {
            Event::MainEventsCleared => {
                //log::info!("MainEventsCleared....");
                application.input(&queue, &input);
                application.update(&device, &queue, &input);
                window.request_redraw();
                input.pre_update();
            }
            Event::RedrawEventsCleared => {
                //log::info!("RedrawEventsCleared....");
                //input.pre_update();
                //application.update(&device, &queue, &input);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    //pool.run_until_stalled();
                    spawner.run_until_stalled();
                }

                //#[cfg(target_arch = "wasm32")]
                //window.request_redraw();
                let close_application = input.key_state(&Key::Q);
                if !close_application.is_none() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::WindowEvent { event, ..} => {
                // Update input cache.
                input.update(&event);

                match event {
                    WindowEvent::Resized(new_size) => {
                        size = new_size;
                        sc_desc.width = new_size.width;
                        sc_desc.height = new_size.height;
                        surface.configure(&device, &sc_desc);
                        //swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        application.resize(&device, &sc_desc, size);
                    }
                    WindowEvent::CloseRequested => {
                        // TODO: application.close()
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }
                //application.input(&queue, &input);
            }
            Event::RedrawRequested(_) => {
                application.render(&device, &mut queue /* &mut swap_chain */, &surface, &sc_desc);
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
        //.with_module_level("wgpu", LevelFilter::Info)
        .init()
        .unwrap();
    }
    //#[cfg(not(target_arch = "wasm32"))]
    //{
    //    let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACE");
    //    subscriber::initialize_default_subscriber(
    //        chrome_tracing_dir.as_ref().map(std::path::Path::new).ok(),
    //    );
    //};

    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);
    #[cfg(windows_OFF)] // TODO
    {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder = builder.with_no_redirection_bitmap(true);
        log::info!("windows_OFF :: True");
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
    // if let Ok(backend) = std::env::var("WGPU_BACKEND") {
    //     log::info!("Backend == {}", backend.to_lowercase().as_str());
    // }
    // let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
    //     match backend.to_lowercase().as_str() {
    //         "vulkan" => wgpu::BackendBit::VULKAN,
    //         "metal" => wgpu::BackendBit::METAL,
    //         "dx12" => wgpu::BackendBit::DX12,
    //         "dx11" => wgpu::BackendBit::DX11,
    //         "gl" =>  wgpu::BackendBit::GL,
    //         "webgpu" =>  wgpu::BackendBit::BROWSER_WEBGPU, 
    //         other => panic!("Unknown backend: {}", other),
    //     }
    // } else {
    //     wgpu::BackendBit::PRIMARY
    // };

    let backend = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY);
    
    let power_preference = if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
        match power_preference.to_lowercase().as_str() {
            "low" => wgpu::PowerPreference::LowPower,
            "high" => wgpu::PowerPreference::HighPerformance,
            other => panic!("Unknown power preference: {}", other),
        }
    } else {
        //wgpu::PowerPreference::default()
        wgpu::PowerPreference::HighPerformance
    };
    log::info!("power_preference = {:?}", power_preference);
    let instance = wgpu::Instance::new(backend);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };
    // let adapter = instance
    //     .request_adapter(&wgpu::RequestAdapterOptions {
    //         power_preference,
    //         compatible_surface: Some(&surface),
    //     })
    //     .await
    //     .expect("No suitable GPU adapters found on the system!");
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
        .await
        .expect("No suitable GPU adapters found on the system!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
    }

    let optional_features = P::optional_features();
    let required_features = P::required_features();
    let adapter_features = adapter.features();
    log::info!("optional_features == {:?}", optional_features);
    log::info!("required_features == {:?}", required_features);
    log::info!("adapter_features == {:?}", adapter_features);
    log::info!("(optional_features & adapter_features) | required_features == {:?}", (optional_features & adapter_features) | required_features);
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );

    let needed_limits = P::required_limits().using_resolution(adapter.limits());

    let trace_dir = std::env::var("WGPU_TRACE");
    log::info!("trace_dir == {:?}", trace_dir);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    // let sc_desc = wgpu::SwapChainDescriptor {
    //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    //     format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
    //     width: size.width,
    //     height: size.height,
    //     present_mode: wgpu::PresentMode::Mailbox,
    // };
      
    let sc_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &sc_desc);
    //let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    Ok(WGPUConfiguration {
            window: window,
            event_loop: event_loop,
            instance: instance,
            size: size,
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            sc_desc: sc_desc,
    })
}

/// Initializes wgpu-rs basic components, application and starts the loop. Native version.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_loop<A: Application, L: Loop, F: WGPUFeatures>() {
    log::info!("Setting up wgpu-rs.");
    let configuration = pollster::block_on(setup::<F>("jihuu")).expect("Failed to create WGPUConfiguration.");
    log::info!("Configurating application.");
    let app = A::init(&configuration);
    log::info!("Initializing loop.");
    let lo = L::init();
    log::info!("Launching the application.");
    lo.run(app, configuration); 
}

/// Initializes wgpu-rs basic components, application and starts the loop. wasm version.
#[cfg(target_arch = "wasm32")]
pub fn run_loop<A: Application, L: Loop, F: WGPUFeatures>() {
    use wasm_bindgen::{prelude::*, JsCast};

//    wasm_bindgen_futures::spawn_local(async move {
//        log::info!("Setting up wgpu-rs.");
//        let configuration = setup::<F>("jihuu").await.unwrap();

        wasm_bindgen_futures::spawn_local(async move {

            log::info!("Setting up wgpu-rs.");
            let configuration = setup::<F>("jihuu").await.unwrap();

            log::info!("Configurating application.");
            let app = A::init(&configuration); 

            log::info!("Creating the application.");
            let lo = L::init();

            ////let setup = setup::<E>(&title).await;
            let start_closure = Closure::once_into_js(move || lo.run(app, configuration));

            // make sure to handle JS exceptions thrown inside start.
            // Otherwise wasm_bindgen_futures Queue would break and never handle any tasks again.
            // This is required, because winit uses JS exception for control flow to escape from `run`.
            if let Err(error) = call_catch(&start_closure) {
                let is_control_flow_exception = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                        e.message().includes("Using exceptions for control flow", 0)
                        });

                if !is_control_flow_exception {
                    web_sys::console::error_1(&error);
                }
            }

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
                fn call_catch(this: &JsValue) -> Result<(), JsValue>;
            }
        });

//        log::info!("Configurating application.");
//        let app = A::init(&configuration); 
//        log::info!("Creating the application.");
//        let lo = L::init();
//        log::info!("Run the application.");
//
//        lo.run(app, configuration);
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> Spawner<'a> {
    pub fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    //#[allow(dead_code)]
    //pub fn spawn_local2(&self, future: impl Future<Output = ()> + 'a) {
    //    self.executor.run(future)
    //}

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
