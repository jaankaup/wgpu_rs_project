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

///////////////////////////////////////////////////////////////////////////////////////

/// A camera for basic rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub pos: cgmath::Vector3<f32>,
    pub view: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov: cgmath::Vector2<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub movement_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

unsafe impl bytemuck::Zeroable for Camera {}
unsafe impl bytemuck::Pod for Camera {}

impl Camera {

    /// TODO: something better.
    pub fn new(aspect_width: f32, aspect_height: f32) -> Self {

        assert!(aspect_height != 0.0, "voeha etta");

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
        let time_delta_milli_f32 = time_delta_nanos as f32 / 1000000.0;

        // println!("time_delta_nanos == {}", time_delta_nanos);
        // println!("movement == {}", (time_delta_nanos as f32) / 1000000.0);
        // println!("********************************");

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

        //println!("movement == ({}, {}, {})", movement.x, movement.y,movement.z);


        match left_mouse_button {
            Some(InputState::Pressed(_)) => { println!("Left mouse is Pressed.") } ,
            Some(InputState::Down(_,_)) => { println!("Left mouse is Down.") } ,
            Some(InputState::Released(_, _)) => { println!("Left mouse is Released.") } ,
            None => { println!("Left mouse is None.") } ,
        }
        
        let md = input_cache.get_mouse_delta();
        // Rotation.
        if let Some(InputState::Down(_,_)) = left_mouse_button {
            println!("mouse_delta == ({}, {})", md.x, md.y);
            self.pitch = clamp(
                self.pitch + (self.rotation_sensitivity as f32 * (md.y * (-1.0)) as f32),
                -89.0,89.0);
            self.yaw = self.yaw + self.rotation_sensitivity * md.x as f32 ;
        }

        println!("camera position == ({}, {}, {})", self.pos.x, self.pos.y, self.pos.z);
        println!("camera view == ({}, {}, {})", self.view.x, self.view.y, self.view.z);
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

///////////////////////////////////////////////////////////////////////////////////////

/// A controller for handling the input and state of camera related operations.
pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_left_mouse_pressed: bool,
    start_mouse_pos: Option<(f64,f64)>,
    current_mouse_pos: Option<(f64,f64)>,
    pub pitch: f32,
    pub yaw: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_left_mouse_pressed: false,
            start_mouse_pos: Some((0 as f64,0 as f64)),
            current_mouse_pos: Some((0 as f64,0 as f64)),
            pitch: -80.5,
            yaw: -50.0,
        }
    }

    pub fn process_events(&mut self, _input_cache: &InputCache) -> bool {
        true    
    }
    
    //pub fn process_events(&mut self, event: &WindowEvent) -> bool {
    //    match event {
    //        WindowEvent::KeyboardInput {
    //            input:
    //                KeyboardInput {
    //                    state,
    //                    virtual_keycode: Some(keycode),
    //                    ..
    //                },
    //            ..
    //        } => {
    //            let is_pressed = *state == ElementState::Pressed;
    //            let event_happened =
    //            match keycode {
    //                VirtualKeyCode::Space => {
    //                    self.is_up_pressed = is_pressed;
    //                    true
    //                }
    //                VirtualKeyCode::C => {
    //                    self.is_down_pressed = is_pressed;
    //                    true
    //                }
    //                VirtualKeyCode::W | VirtualKeyCode::Up => {
    //                    self.is_forward_pressed = is_pressed;
    //                    true
    //                }
    //                VirtualKeyCode::A | VirtualKeyCode::Left => {
    //                    self.is_left_pressed = is_pressed;
    //                    true
    //                }
    //                VirtualKeyCode::S | VirtualKeyCode::Down => {
    //                    self.is_backward_pressed = is_pressed;
    //                    true
    //                }
    //                VirtualKeyCode::D | VirtualKeyCode::Right => {
    //                    self.is_right_pressed = is_pressed;
    //                    true
    //                }
    //                _ => false,
    //            };

    //            event_happened // TODO: remove

    //        }, // WindowEvent::KeyboardInput

    //        WindowEvent::MouseInput {
    //                state,
    //                button,
    //                ..
    //        } => { 
    //            let is_pressed = *state == ElementState::Pressed;
    //            let event_happened =
    //            match button {
    //                MouseButton::Left => {
    //                    self.is_left_mouse_pressed = is_pressed;
    //                    true
    //                }
    //                _ => false,
    //            };

    //            event_happened
    //        }, // WindowEvent::MouseEvent

    //        WindowEvent::CursorMoved {
    //                position,
    //                ..
    //        } => { 

    //            // Initial mouse positions.
    //            match self.start_mouse_pos {
    //                Some(_) => { },
    //                None      => {
    //                    self.start_mouse_pos = Some((position.x, position.y));
    //                    self.current_mouse_pos = Some((position.x, position.y));
    //                },
    //            }

    //            // Update both previous and current mouse positions.
    //            self.start_mouse_pos = self.current_mouse_pos;
    //            self.current_mouse_pos = Some((position.x, position.y));

    //            true
    //        }, // WindowEvent::CursorMoved

    //        _ => false, // ignore other events
    //    } // event
    //} // end func

    pub fn update_camera(&mut self, camera: &mut Camera, input_cache: &InputCache) {
                                                               
        let forward = camera.view;
        //println!("{}", input_cache.key_state(&Key::W) == false);

        let state: Option<InputState> = input_cache.key_state(&Key::W);

        match state {
            Some(InputState::Down(_,_)) => { println!("Moving forward (down).") }, 
            Some(InputState::Pressed(_)) => { println!("Moving forward (pressed).") }, 
            Some(InputState::Released(_,_)) => { println!("Stop moving forward (released)") }, 
            _ => { println!("Released long time ago.") },
            //camera.pos += forward * self.speed;
        }
         //if let input_cache.key_state(&Key::W) {
         //    camera.pos += forward * self.speed;
         //}
        if self.is_backward_pressed {
            camera.pos -= forward * self.speed;
        }

        let right = forward.cross(camera.up);

        if self.is_right_pressed {
            camera.pos += right * self.speed;
        }
        if self.is_left_pressed {
            camera.pos -= right * self.speed;
        }
        if self.is_up_pressed {
            camera.pos += camera.up * self.speed;
        }
        if self.is_down_pressed {
            camera.pos -= camera.up * self.speed;
        }
        if self.is_left_mouse_pressed {
            // Update mouse delta.
            let (x0, y0) = self.start_mouse_pos.unwrap();
            let (x1, y1) = self.current_mouse_pos.unwrap();
            let (x,y) = (x1 - x0, y1 - y0); 

            self.pitch = clamp(self.pitch + (self.sensitivity as f32 * (y * (-1.0)) as f32) , -89.0,89.0);
            //self.pitch = clamp(self.pitch + (self.sensitivity as f32 * (y * (-1.0)) as f32) , -89.0,89.0);
            self.yaw = self.yaw + self.sensitivity * x as f32 ;

            // println!("yaw/pitch = ({},{})", self.yaw, self.pitch);

            camera.view = Vector3::new(
                self.pitch.to_radians().cos() * self.yaw.to_radians().cos(),
                self.pitch.to_radians().sin(),
                self.pitch.to_radians().cos() * self.yaw.to_radians().sin()
            ).normalize_to(1.0);

            // println!("view = ({},{},{})", camera.view.x, camera.view.y, camera.view.z);

        }
    }

    //TODO: refactor.
    pub fn update_ray_camera(&mut self, camera: &mut RayCamera) {
                                                               
        let forward = camera.view;

        if self.is_forward_pressed {
            camera.pos += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.pos -= forward * self.speed;
        }

        let right = forward.cross(camera.up);

        if self.is_right_pressed {
            camera.pos += right * self.speed;
        }
        if self.is_left_pressed {
            camera.pos -= right * self.speed;
        }
        if self.is_up_pressed {
            camera.pos += camera.up * self.speed;
        }
        if self.is_down_pressed {
            camera.pos -= camera.up * self.speed;
        }
        if self.is_left_mouse_pressed {
            // Update mouse delta.
            let (x0, y0) = self.start_mouse_pos.unwrap();
            let (x1, y1) = self.current_mouse_pos.unwrap();
            let (x,y) = (x1 - x0, y1 - y0); 

            self.pitch = clamp(self.pitch + (self.sensitivity as f32 * (y * (-1.0)) as f32) , -89.0,89.0);
            self.yaw = self.yaw + self.sensitivity * x as f32 ;

            // println!("yaw/pitch = ({},{})", self.yaw, self.pitch);

            camera.view = Vector3::new(
                self.pitch.to_radians().cos() * self.yaw.to_radians().cos(),
                self.pitch.to_radians().sin(),
                self.pitch.to_radians().cos() * self.yaw.to_radians().sin()
            ).normalize_to(1.0);

            // println!("ray_view = ({},{},{})", camera.view.x, camera.view.y, camera.view.z);

        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////

///// TODO: remove this. Add to the RayCamera. 
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

/// Camera uniform data for the shader.
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
