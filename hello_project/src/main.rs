use jaankaup_core::wgpu_system;
//#[path = "../../wgpu_system.rs"]
//mod wgpu_system;
//
//#[path = "../../application.rs"]
//mod application;

/// A struct for features and limits used in this application.
struct MyFeatures {}
impl wgpu_system::WGPUFeatures for MyFeatures { }

struct TestApp {

}

fn main() {
    
    println!("Started...");
    let jaahans = futures::executor::block_on(wgpu_system::setup::<MyFeatures>("jihuu"));
    println!("Finished...");
}
