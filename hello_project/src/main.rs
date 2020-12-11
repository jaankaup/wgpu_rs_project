use jaankaup_core::wgpu_system as ws;
use jaankaup_core::wgpu_system::{
        WGPUFeatures,
        WGPUConfiguration,
        Application,
        BasicLoop
};

// Redefine needed features for this application.
struct MyFeatures {}
impl WGPUFeatures for MyFeatures { 
}

// State for this application.
struct HelloApp {
}

impl Application for HelloApp {

    fn init(_configuration: &WGPUConfiguration) -> Self {
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

fn main() {
    
    ws::run_loop::<HelloApp, BasicLoop, MyFeatures>(); 
    println!("Finished...");
}
