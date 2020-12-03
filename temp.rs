mod wgpu_system;

#[cfg(target_arch = "wasm32")]
use futures::task::LocalSpawn;

use winit::{
    event::{Event, WindowEvent,KeyboardInput,ElementState,VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

//pub trait WGPUSystem: 'static + Sized {

struct App {

}

/// A struct for features and limits used in this application.
struct MyFeatures {}

impl wgpu_system::WGPUFeatures for MyFeatures {
    // Default features. TODO: specify custom features/limits.
}

//impl App {
//    pub async fn new(window: &Window) -> Self {
//        App { name: "joo".to_string(),}
//    }
//}

// TODO: setup function :: Create wgpu-rs.
// TODO: run function :: call setup and start.
// TODO: start function :: create pool, spawner, start event-loop
// TODO: InputCache
// TODO: Graphics module (for wgpu-rs)
// TODO: Import camera/controller

/// A function that initializes spawner and pool. Then event loop is executed.
fn run(window: Window, event_loop: EventLoop<()>, mut state: App) {

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


#[cfg(not(target_arch = "wasm32"))]
pub fn run_app<P: wgpu_system::WGPUFeatures>(app_name: &str) {
    let title = app_name.to_owned();
    println!("YEAH");

    // Create the initial wgpu-rs resources.
    let setup = futures::executor::block_on(wgpu_system::setup::<P>(&title));
}

#[cfg(target_arch = "wasm32")]
pub fn run_app() {

}


fn main() {

    //println!("Started...");
    //let jaahans = futures::executor::block_on(wgpu_system::setup::<MyFeatures>("jihuu"));
    //println!("Finished...");

    //let event_loop = EventLoop::new();
    //let mut builder = winit::window::WindowBuilder::new();
    //builder = builder.with_title("My wgpu-rs project");
    //let window = builder.build(&event_loop).unwrap();
    ////let window = winit::window::Window::new(&event_loop).unwrap();

    //// Run the native version.
    //#[cfg(not(target_arch = "wasm32"))]
    //{
    //    let state = futures::executor::block_on(App::new(&window));
    //    run(window, event_loop, state);
    //}

    //// Configure and run wasm version.
    //#[cfg(target_arch = "wasm32")]
    //{
    //    // Add logging system for wasm.
    //    console_log::init().expect("could not initialize logger");
    //    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    //    use winit::platform::web::WindowExtWebSys;

    //    // On wasm, append the canvas to the document body
    //    // I guess this is the place to add all the web-stuff.
    //    web_sys::window()
    //        .and_then(|win| win.document())
    //        .and_then(|doc| doc.body())
    //        .and_then(|body| {
    //            body.append_child(&web_sys::Element::from(window.canvas()))
    //                .ok()
    //        })
    //        .expect("couldn't append canvas to document body");

    //    // Run the wasm version.
    //    wasm_bindgen_futures::spawn_local(async move {let mut state = App::new(&window).await; run(window, event_loop, state);});
    //}
}
