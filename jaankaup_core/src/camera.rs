use crate::misc::clamp;
use crate::input::{InputCache, InputState};
//use cgmath::{prelude::*};
use cgmath::{prelude::*, Vector3, Vector4, Point3};
//use winit::{
//    event::{WindowEvent,KeyboardInput,ElementState,VirtualKeyCode,MouseButton},
//};

pub use winit::event::VirtualKeyCode as Key;
pub use winit::event::MouseButton as MouseButton;

/// Opengl to wgpu matrix
#[cfg_attr(rustfmt, surtfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
);

/// A camera for ray tracing purposes.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RayCamera {
    pub pos: cgmath::Vector3<f32>,
    pub view: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub fov: cgmath::Vector2<f32>,
    pub aperture_radius: f32,
    pub focal_distance: f32,
}

unsafe impl bytemuck::Zeroable for RayCamera {}
unsafe impl bytemuck::Pod for RayCamera {}

/// Struct that represent uniform data in shader.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUniform {
    view_proj: cgmath::Matrix4<f32>,
    pos: cgmath::Vector4<f32>,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity(),
            pos: cgmath::Vector4::new(1.0,1.0,1.0,1.0),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_projection_matrix();
        self.pos = Vector4::new(camera.pos.x, camera.pos.y, camera.pos.z, 1.0);
    }
}
 
unsafe impl bytemuck::Zeroable for CameraUniform {}
unsafe impl bytemuck::Pod for CameraUniform {}

/// A camera for basic rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pos: cgmath::Vector3<f32>,
    view: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fov: cgmath::Vector2<f32>,
    znear: f32,
    zfar: f32,
    movement_sensitivity: f32,
    rotation_sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

unsafe impl bytemuck::Zeroable for Camera {}
unsafe impl bytemuck::Pod for Camera {}

impl Camera {

    /// TODO: something better.
    pub fn new(aspect_width: f32, aspect_height: f32) -> Self {

        assert!(aspect_height > 0.0, "Height must be > 0.");
        assert!(aspect_width > 0.0, "Width must be > 0.");

        Self {
            pos: (1.0, 1.0, 1.0).into(),
            view: Vector3::new(0.0, 0.0, -1.0).normalize(),
            up: cgmath::Vector3::unit_y(),
            aspect: aspect_width / aspect_height as f32,
            fov: (45.0,45.0).into(),
            znear: 0.01,
            zfar: 1000.0,
            movement_sensitivity: 0.5,
            rotation_sensitivity: 0.5,
            pitch: -80.5,
            yaw: -50.5,
        }
    }

    /// Update camera from user input.
    pub fn update_from_input(&mut self, input_cache: &InputCache) {

        // Get the keyboard state (camera movement).
        let state_forward = input_cache.key_state(&Key::W);
        let state_backward = input_cache.key_state(&Key::S);
        let state_right = input_cache.key_state(&Key::D);
        let state_left = input_cache.key_state(&Key::A);
        let state_up = input_cache.key_state(&Key::E);
        let state_down = input_cache.key_state(&Key::C);
        let left_mouse_button = input_cache.mouse_button_state(&MouseButton::Left);

        // Get the delta time between previous and current tick.
        let time_delta_nanos = input_cache.get_time_delta();

        // Convert time delta to milli seconds.
        let time_delta_milli_f32 = time_delta_nanos as f32 / 1000000.0;

        // The right vector.
        let right = self.view.cross(self.up);

        let mut movement = cgmath::Vector3::new(0.0, 0.0, 0.0);

        // Moving forward. Moving forward if forward key is pressed, down or released. 
        if !state_forward.is_none() { movement += time_delta_milli_f32 * self.view; }
        if !state_backward.is_none() { movement -= time_delta_milli_f32 * self.view; }
        if !state_right.is_none() { movement += time_delta_milli_f32 * right; }
        if !state_left.is_none() { movement -= time_delta_milli_f32 * right; }
        if !state_up.is_none() { movement += time_delta_milli_f32 * self.up; }
        if !state_down.is_none() { movement -= time_delta_milli_f32 * self.up; }

        self.pos += movement;


        // Rotation.
          
        let md = input_cache.get_mouse_delta();

        // If left mouse is down update pitch, yaw and view.
        if let Some(InputState::Down(_,_)) = left_mouse_button {

            self.pitch = clamp(
                self.pitch + (self.rotation_sensitivity as f32 * (md.y * (-1.0)) as f32),
                -89.0,89.0);
            self.yaw = self.yaw + self.rotation_sensitivity * md.x as f32 ;

            self.view = Vector3::new(
                self.pitch.to_radians().cos() * self.yaw.to_radians().cos(),
                self.pitch.to_radians().sin(),
                self.pitch.to_radians().cos() * self.yaw.to_radians().sin()
            ).normalize_to(1.0);
        }
    }

    /// Creates a pv matrix for wgpu.
    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {

        let view = self.build_view_matrix();
        let proj = cgmath::perspective(cgmath::Rad(std::f32::consts::PI/2.0), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * (proj * view)
    }

    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let pos3 = Point3::new(self.pos.x, self.pos.y,self.pos.z);
        let view3 = Point3::new(self.view.x + pos3.x, self.view.y + pos3.y, self.view.z + pos3.z);
        let view = cgmath::Matrix4::look_at(pos3, view3, self.up);
        view
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RayCameraUniform {
    pos: cgmath::Vector4<f32>,  // eye
    view: cgmath::Vector4<f32>, // target    // original: float3
    up: cgmath::Vector4<f32>,
    fov: cgmath::Vector4<f32>, // fovy
    aperture_radius: f32, // new!
    focal_distance: f32, // new!
}

impl RayCameraUniform {
    pub fn new() -> Self {
        Self {
            pos: (1.0, 1.0, 1.0, 1.0).into(),
            view: Vector4::new(0.0, 0.0, -1.0, 0.0).normalize(),
            up: cgmath::Vector4::unit_y(),
            fov: ((45.0 as f32).to_radians(),
                 (45.0 as f32).to_radians(),
                 111.0,
                 222.0).into(),
            aperture_radius: 0.0,
            focal_distance: 1.0,
        }
    }

    pub fn update(&mut self, camera: &RayCamera) {
            self.pos  = cgmath::Vector4::new(camera.pos.x, camera.pos.y,  camera.pos.z, 1.0);  
            self.view = cgmath::Vector4::new(camera.view.x, camera.view.y, camera.view.z, 0.0);
            self.up   = cgmath::Vector4::new(camera.up.x, camera.up.y,   camera.up.z, 0.0);  
            self.fov  = cgmath::Vector4::new(camera.fov.x, camera.fov.y, 123.0, 234.0); // 2 dummy values. 
            self.aperture_radius = camera.aperture_radius;
            self.focal_distance = camera.focal_distance;
    }
}

unsafe impl bytemuck::Zeroable for RayCameraUniform {}
unsafe impl bytemuck::Pod for RayCameraUniform {}

///////////////////////////////////////////////////////////////////////////////////////

