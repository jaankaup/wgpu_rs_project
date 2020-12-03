use jaankaup_core::wgpu_system as ws;
use crate::ws::BasicLoop;
use jaankaup_core::wgpu_system::Application;
use jaankaup_core::wgpu_system::Loop;
//use jaankaup_core::wgpu_system::Application;

struct MyFeatures {}
impl ws::WGPUFeatures for MyFeatures { 
}

struct HelloApp {
}

impl ws::Application for HelloApp {

    fn init(configuration: &ws::WGPUConfiguration) -> Self {
        HelloApp { }
    }

    fn render(self) {

    }

    fn input(self) {

    }

    fn resize(self) {

    }

    fn update(self) {

    }
}

// pub trait Application: Sized + 'static {
// 
//     /// Creates an Application.
//     fn init(self, configuration: &WGPUConfiguration) -> Self;
// 
//     /// The render function for application.
//     fn render(self);
// 
//     /// A function that handles inputs.
//     fn input(self);
// 
//     /// A function for resizing.
//     fn resize(self);
// 
//     /// A function for updating the state of the application.
//     fn update(self);
// }

// /// A trait for Loops.
// pub trait Loop: Sized + 'static {
// 
//     /// Run function that starts the loop.
//     fn run<A: Application>(self, configuration: WGPUConfiguration);
// }

struct TestApp {

}

fn main() {
    
    println!("Started...");
    #[cfg(not(target_arch = "wasm32"))] {
        let configuration = futures::executor::block_on(ws::setup::<MyFeatures>("jihuu")).expect("Failed to create WGPUConfiguration.");
        let app = HelloApp::init(&configuration);
        let basic_loop = BasicLoop {};
        basic_loop.run(app, configuration); 
    }

    #[cfg(target_arch = "wasm32")]
    {
        //use futures::{future::LocalFutureObj, task::SpawnError};
        wasm_bindgen_futures::spawn_local(async move {
            let configuration = ws::setup::<MyFeatures>("jihuu").await.unwrap();
            let app = HelloApp::init(&configuration); 
            let basic_loop = BasicLoop {};
            //basic_loop<HelloApp>(application: A, WGPUConfiguration {
            basic_loop.run(app, configuration); 
        });
    }


    println!("Finished...");
}
