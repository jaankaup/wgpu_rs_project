use std::collections::HashMap;
use std::time::{Duration, Instant};

use winit::event as ev;

pub use ev::VirtualKeyCode as Key;
use winit::dpi::PhysicalPosition;

pub enum InputState {
    Pressed,
    Down(u64),
    Reseased(u64),
}

/// A stuct for input handling. The idea is obtained from https:/github.com/MoleTrooper/starframe.
pub struct InputCache {
    keyboard: HashMap<Key, InputState>,
}

impl InputCache {
    /// Initialize InputCache.
    pub fn init() -> Self {
        let keyboard = HashMap::<Key, InputState>::with_capacity(128);
        let timer = Instant::now(); 

        Self {
            keyboard: keyboard,
        }
    }
    /// Process the new inputs.
    pub fn update(&mut self, event: &ev::WindowEvent) {
        use ev::WindowEvent::*;
        match event {
            KeyboardInput { input, ..} => self.track_keyboard(*input),
            MouseInput { button, state, ..} => self.track_mouse_button(*button, *state),
            MouseWheel { delta, ..} => self.track_mouse_wheel(*delta),
            CursorMoved { position, ..} => self.track_cursor_movement(*position),
            CursorEntered { ..} => self.track_cursor_enter(),
            CursorLeft { ..} => self.track_cursor_leave(),
            _ => (),
        }
    }
    /// Get the state of keyboard key.
    pub fn key_state(key: &Key) -> Option<InputState> {
        None
    }

    /// Update the state of keyboard.
    fn track_keyboard(&mut self, evt: ev::KeyboardInput) {
    
    }
    /// Update the state of mouse buttons.
    fn track_mouse_button(&mut self, button: ev::MouseButton, state: ev::ElementState) {

    }
    /// Update the state of mouse wheel.
    fn track_mouse_wheel(&mut self, delta: ev::MouseScrollDelta) {

    }
    /// Update the state of mouse movement.
    fn track_cursor_movement(&mut self, position: PhysicalPosition<f64>) {

    }
    /// Handle the cursor enter event. TODO: implement.
    fn track_cursor_enter(&mut self) {
        unimplemented!();
    }
    /// Handle the cursor leave event. TODO: implement.
    fn track_cursor_leave(&mut self) {
        unimplemented!();
    }
}
