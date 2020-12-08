use std::collections::HashMap;
use std::time::{Duration, Instant};

use winit::event as ev;

pub use ev::VirtualKeyCode as Key;
use winit::dpi::PhysicalPosition;

/// An enum for mouse and keyboard button states.
#[derive(Clone)]
pub enum InputState {
    Pressed,
    Down(u64),
    Reseased(u64),
}

/// A struct for mouse buttons.
#[derive(Clone)]
pub struct MouseButtons {
    left: Option<InputState>,
    middle: Option<InputState>,
    right: Option<InputState>,
}

impl MouseButtons {
    pub fn init() -> Self {
        Self {
            left: None,
            middle: None,
            right: None,
        }
    }
    pub fn get_left(self) -> Option<InputState> {
        self.left
    }
    pub fn get_middle(self) -> Option<InputState> {
        self.middle
    }
    pub fn get_right(self) -> Option<InputState> {
        self.right
    }
}

/// A stuct for holdin information about the mouse cursor position.
#[derive(Clone, Copy)]
pub struct CursorPosition {
    pos: Option<PhysicalPosition<f64>>,
    inside: bool,
}

impl CursorPosition {
    pub fn init() -> Self {
        Self {
            pos: None,
            inside: false,
        }
    }
}

/// A stuct for input handling. The idea is derived from https:/github.com/MoleTrooper/starframe.
#[derive(Clone)]
pub struct InputCache {
    pub keyboard: HashMap<Key, InputState>,
    pub mouse_buttons: MouseButtons,
    pub mouse_position: CursorPosition,
    pub mouse_delta: PhysicalPosition::<f64>,
    pub scroll_delta: f32,
    pub time_now: u128,
    pub time_delta: u128,
    pub timer: Instant,
}

impl InputCache {
    /// Initialize InputCache.
    pub fn init() -> Self {
        let keyboard = HashMap::<Key, InputState>::with_capacity(128);
        let mouse_buttons = MouseButtons::init();
        let mouse_position = CursorPosition::init();
        let timer = Instant::now(); 

        Self {
            keyboard: keyboard,
            mouse_buttons: mouse_buttons,
            mouse_position: mouse_position,
            mouse_delta: PhysicalPosition::<f64>::new(0.0, 0.0),
            scroll_delta: 0.0,
            time_now: 0,
            time_delta: 0,
            timer: timer,
        }
    }
    /// Process the new inputs.
    pub fn update(&mut self, event: &ev::WindowEvent) {
        use ev::WindowEvent::*;

        // Update timer.
        let now = self.timer.elapsed().as_nanos();
        self.time_delta = now - self.time_now;
        self.time_now = now;
        println!("Time delta == {}", self.time_delta);

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
        println!("track_keyboard");
    }
    /// Update the state of mouse buttons.
    fn track_mouse_button(&mut self, button: ev::MouseButton, state: ev::ElementState) {
        println!("track_mouse_button");
    }
    /// Update the state of mouse wheel.
    fn track_mouse_wheel(&mut self, delta: ev::MouseScrollDelta) {
        println!("track_mouse_wheel");
    }
    /// Update the state of mouse movement.
    fn track_cursor_movement(&mut self, new_pos: PhysicalPosition<f64>) {
        println!("track_cursor_movement");
        match self.mouse_position.pos {
            None => { self.mouse_position.pos = Some(new_pos); }
            Some(old_position) => {
                self.mouse_delta = PhysicalPosition::<f64>::new(new_pos.x - old_position.x , new_pos.y - old_position.y);
                self.mouse_position.pos = Some(new_pos);
            }
        }
        println!("mouse_delta = ({}, {})", self.mouse_delta.x, self.mouse_delta.y);
    }
    /// Handle the cursor enter event. TODO: implement.
    fn track_cursor_enter(&mut self) {
        println!("track_cursor_enter");
    }
    /// Handle the cursor leave event. TODO: implement.
    fn track_cursor_leave(&mut self) {
        self.mouse_delta = PhysicalPosition::<f64>::new(0.0, 0.0);
        println!("track_cursor_leave");
    }
}
