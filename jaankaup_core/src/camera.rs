use crate::misc::clamp;
use crate::input::{InputCache, InputState};
use crate::buffer::buffer_from_data;
use cgmath::{prelude::*, Vector3, Vector4, Point3};

pub use winit::event::VirtualKeyCode as Key;
pub use winit::event::MouseButton as MouseButton;

/// Opengl to wgpu matrix
//#[cfg_attr(rustfmt, surtfmt_skip)]
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
);

/// Struct that represent camera uniform data in shader. The projection matrix and the position of
/// the camera.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUniform {
    view_proj: cgmath::Matrix4<f32>,
    pos: cgmath::Vector4<f32>,
}

unsafe impl bytemuck::Zeroable for CameraUniform {}
unsafe impl bytemuck::Pod for CameraUniform {}

/// Struct that represent ray tracing camera uniform data in shader.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RayCameraUniform {
    pos: cgmath::Vector4<f32>,
    view: cgmath::Vector4<f32>,
    up: cgmath::Vector4<f32>,
    fov: cgmath::Vector2<f32>,
    aperture_radius: f32,
    focal_distance: f32,
}

unsafe impl bytemuck::Zeroable for RayCameraUniform {}
unsafe impl bytemuck::Pod for RayCameraUniform {}

/// A camera for basic rendering and ray tracing purposes.
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
    aperture_radius: f32, // For ray tracer camera.
    focal_distance: f32, // For ray tracer camera.
    camera_buffer: Option<wgpu::Buffer>, // A buffer to basic camera uniform buffer.
    ray_camera_buffer: Option<wgpu::Buffer>, // A buffer to ray tracing camear uniform buffer.
}

impl Camera {

    pub fn set_movement_sensitivity(&mut self, sensitivity: f32) {
        assert!(sensitivity > 0.0, "Movement sensitivity must be > 0.");
        self.movement_sensitivity = sensitivity;
    }

    pub fn set_rotation_sensitivity(&mut self, sensitivity: f32) {
        assert!(sensitivity > 0.0, "Rotation sensitivity must be > 0.");
        self.rotation_sensitivity = sensitivity;
    }

    /// Get a reference to camera uniform buffer. Creates the buffer is it doens't already exist.
    pub fn get_camera_uniform(&mut self, device: &wgpu::Device) -> &wgpu::Buffer {

        // Create camera uniform data.
        let camera_uniform = CameraUniform {
            view_proj: self.build_projection_matrix(),
            pos: Vector4::new(self.pos.x, self.pos.y, self.pos.z, 1.0),
        };

        // The camera uniform buffer doesn't exist. Create camera buffer.
        if self.camera_buffer.is_none() {

            self.camera_buffer = Some(buffer_from_data::<CameraUniform>(
                &device,
                &[camera_uniform],
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                None)
            );
        }

        &self.camera_buffer.as_ref().unwrap()
    }
    
    // TODO: update uniform?
    pub fn resize(&mut self, aspect_width: f32, aspect_height: f32) {
        self.aspect = aspect_width / aspect_height as f32;
    }

    /// Get a reference to ray tracing camera uniform buffer. Creates the buffer is it doens't already exist.
    /// TODO: create buffer on init().
    pub fn get_ray_camera_uniform(&mut self, device: &wgpu::Device) -> &wgpu::Buffer {

        // Create ray camera uniform data.
        let ray_camera_uniform = RayCameraUniform {
            pos: cgmath::Vector4::<f32>::new(self.pos.x, self.pos.y, self.pos.z, 1.0),
            view: cgmath::Vector4::<f32>::new(self.view.x, self.view.y, self.view.z, 0.0),
            up: cgmath::Vector4::<f32>::new(self.up.x, self.up.y, self.up.z, 0.0),
            fov: self.fov,
            aperture_radius: self.aperture_radius,
            focal_distance: self.focal_distance,
        };

        // The ray camera uniform buffer doesn't exist. Create ray camera buffer.
        if self.ray_camera_buffer.is_none() {

            self.ray_camera_buffer = Some(buffer_from_data::<RayCameraUniform>(
                &device,
                &[ray_camera_uniform],
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                None)
            );
        }

        &self.ray_camera_buffer.as_ref().unwrap()
    }

    /// TODO: something better.
    pub fn new(aspect_width: f32, aspect_height: f32) -> Self {

        assert!(aspect_height > 0.0, "Height must be > 0.");
        assert!(aspect_width > 0.0, "Width must be > 0.");

        Self {
            pos: (3.0, 4.0, 1.0).into(),
            view: Vector3::new(0.0, 0.0, -1.0).normalize(),
            up: cgmath::Vector3::unit_y(),
            aspect: aspect_width / aspect_height as f32,
            fov: (45.0,45.0).into(),
            znear: 0.01,
            zfar: 1000.0,
            movement_sensitivity: 0.003,
            rotation_sensitivity: 0.05,
            pitch: -80.5,
            yaw: -50.5,
            aperture_radius: 0.01,
            focal_distance: 1.0,
            camera_buffer: None,
            ray_camera_buffer: None,
        }
    }

    /// Update camera from user input. TODO: create a method for 
    /// Bezier-curvers and B-splines.
    pub fn update_from_input(&mut self, queue: &wgpu::Queue, input_cache: &InputCache) {

        // Get the keyboard state (camera movement).
        let state_forward = input_cache.key_state(&Key::W);
        let state_backward = input_cache.key_state(&Key::S);
        let state_right = input_cache.key_state(&Key::D);
        let state_left = input_cache.key_state(&Key::A);
        let state_up = input_cache.key_state(&Key::E);
        let state_down = input_cache.key_state(&Key::C);
        let left_mouse_button = input_cache.mouse_button_state(&MouseButton::Left);
        let left_shift = input_cache.key_state(&Key::LShift);

        // Get the delta time between previous and current tick.
        let time_delta_nanos = input_cache.get_time_delta();

        // Convert time delta to milli seconds.
        let time_delta_milli_f32 = time_delta_nanos as f32 / 1000000.0;

        // The right vector.
        let right = self.view.cross(self.up);

        let mut movement = cgmath::Vector3::new(0.0, 0.0, 0.0);

        // 1/10 speed if left shift is down.
        let mut movement_factor = 1.0;
        if !left_shift.is_none() { movement_factor = 0.1; }

        // Calculate the amount of movement based on user input.
        if !state_forward.is_none() { movement += movement_factor * time_delta_milli_f32 * self.view; }
        if !state_backward.is_none() { movement -= movement_factor * time_delta_milli_f32 * self.view; }
        if !state_right.is_none() { movement += movement_factor * time_delta_milli_f32 * right; }
        if !state_left.is_none() { movement -= movement_factor * time_delta_milli_f32 * right; }
        if !state_up.is_none() { movement += movement_factor * time_delta_milli_f32 * self.up; }
        if !state_down.is_none() { movement -= movement_factor * time_delta_milli_f32 * self.up; }

        // Update the camera position.
        self.pos += self.movement_sensitivity * movement;

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

        // Update the camera uniform and the camera uniform buffer.
        if !self.camera_buffer.is_none() {

            // Create camera uniform data. TODO: refactor.
            let camera_uniform = CameraUniform {
                view_proj: self.build_projection_matrix(),
                pos: Vector4::new(self.pos.x, self.pos.y, self.pos.z, 1.0),
            };
            queue.write_buffer(
                &self.camera_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[camera_uniform])
            );
        }
    }

    /// Creates a pv matrix for wgpu.
    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {

        let view = self.build_view_matrix();
        let proj = cgmath::perspective(cgmath::Rad(std::f32::consts::PI/2.0), self.aspect, self.znear, self.zfar);

        // Convert "opengl" matrix to wgpu matris.
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    /// Build view projection matrix.
    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let pos3 = Point3::new(self.pos.x, self.pos.y,self.pos.z);
        let view3 = Point3::new(self.view.x + pos3.x, self.view.y + pos3.y, self.view.z + pos3.z);
        let view = cgmath::Matrix4::look_at_rh(pos3, view3, self.up);
        view
    }
}
