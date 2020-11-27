use futures::task::LocalSpawn;
use winit::{
    event::{Event, WindowEvent,KeyboardInput,ElementState,VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

struct App {
    name: String,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        App { name: "joo".to_string(),}
    }
}



/// A function that set up things and starts the event loop.
fn run(window: Window, event_loop: EventLoop<()>, mut state: App) {

    #[cfg(all(not(target_arch = "wasm32"), feature = "subscriber"))]
    {
        let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACING");
        wgpu::util::initialize_default_subscriber(chrome_tracing_dir.as_ref().map(std::path::Path::new).ok());
    };

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

        //std::panic::set_hook(Box::new(console_error_panic_hook::hook));

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

    event_loop.run(move |event, _, control_flow| {

        // Move the ownership of state and window to the 
        let _ = (&state,&window);
        *control_flow = ControlFlow::Poll;
        #[cfg(not(target_arch = "wasm32"))]
        {
            pool.run_until_stalled();
        }

        match event {

            Event::MainEventsCleared => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    pool.run_until_stalled();
                }

                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                //state.resize(size);
            }
            Event::WindowEvent {event, .. } => {
                //if state.input(&event) { /* state.update() */ }
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::KeyboardInput { input, ..  } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        } // match input
                    } // KeyboardInput
                    _ => { /*state.update()*/ } // Other WindowEvents
                } // match event (WindowEvent)
            } // Event::WindowEvent
            Event::RedrawRequested(_) => {
                //state.render(&window);
            }
            _ => { } // Any other events
        } // match event
    }); // run
}

fn main() {

    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("My wgpu-rs project");
    let window = builder.build(&event_loop).unwrap();
    //let window = winit::window::Window::new(&event_loop).unwrap();

    // Run the native version.
    #[cfg(not(target_arch = "wasm32"))]
    {
        let state = futures::executor::block_on(App::new(&window));
        run(window, event_loop, state);
    }

    // Configure and run wasm version.
    #[cfg(target_arch = "wasm32")]
    {
        // Add logging system.
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");

        use winit::platform::web::WindowExtWebSys;

        // On wasm, append the canvas to the document body
        // I guess this is the place to add all the web-stuff.
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Run the wasm version.
        wasm_bindgen_futures::spawn_local(async move {let mut state = App::new(&window).await; run(window, event_loop, state);});
    }
}
