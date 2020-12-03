#[cfg(target_arch = "wasm32")]
use futures::task::LocalSpawn;

use jaankaup_core::wgpu_system as ws;

struct MyFeatures {}
impl ws::WGPUFeatures for MyFeatures { }

struct Hello_app {
}

impl ws::Application for Hello_app  {

    fn init(self, configuration: &ws::WGPUConfiguration) -> Self {
        Hello_app {
        }
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
    let configuration = futures::executor::block_on(ws::setup::<MyFeatures>("jihuu")).expect("Failed to create WGPUConfiguration.");


    println!("Finished...");
}
