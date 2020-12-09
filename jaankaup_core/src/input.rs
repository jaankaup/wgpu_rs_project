use std::collections::HashMap;
use std::time::{Duration, Instant};

use winit::event as ev;

pub use ev::VirtualKeyCode as Key;
use winit::dpi::PhysicalPosition;

/// An enum for mouse and keyboard button states.
#[derive(Clone,Debug)]
pub enum InputState {
    Pressed(u128),
    Down(u128,u128),
    Released(u128, u128),
}

impl InputState {
    /// Updates the state depending on given ElementState and the current state.
    pub fn update(&mut self, state: &ev::ElementState, time_now: u128) {
        match state {
            ev::ElementState::Pressed => {
                match std::mem::replace(self, InputState::Pressed(666)) {
                    InputState::Pressed(start_time) => {
                        *self = InputState::Down(start_time,time_now)
                    }
                    InputState::Down(start_time, _) => {
                        *self = InputState::Down(start_time,time_now)
                    }
                    InputState::Released(_,_) => {
                        *self = InputState::Pressed(time_now)
                    }
                }
            }
            ev::ElementState::Released => {
                match std::mem::replace(self, InputState::Pressed(666)) {
                    InputState::Pressed(start_time) => {
                        *self = InputState::Released(start_time,time_now)
                    }
                    InputState::Down(start_time, _) => {
                        *self = InputState::Released(start_time,time_now)
                    }
                    InputState::Released(_,_) => {
                        panic!("Already released.")
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MouseButton {
    state: Option<InputState>,
    tag: ev::MouseButton,
}

/// A struct for mouse buttons.
#[derive(Clone)]
pub struct MouseButtons {
    left: MouseButton,
    middle: MouseButton,
    right: MouseButton,
}

impl MouseButtons {
    pub fn init() -> Self {
        Self {
            left: MouseButton   { state: None , tag: ev::MouseButton::Left},
            middle: MouseButton { state: None , tag: ev::MouseButton::Middle},
            right: MouseButton  { state: None , tag: ev::MouseButton::Right},
        }
    }
    pub fn update(&mut self, button: &ev::MouseButton, state: &ev::ElementState, time_now: u128) {
        match button {
            ev::MouseButton::Left => {
                match &mut self.left.state {
                    Some(s) => {
                        s.update(&state, time_now);
                        println!("Left mouse : {:?}", self.left.state.as_ref());
                    }
                    None => {
                        self.left.state = Some(InputState::Pressed(time_now));
                        println!("Left mouse : {:?}", self.left.state.as_ref());
                    }
                }
            }
            ev::MouseButton::Middle => {
                match &mut self.middle.state {
                    Some(s) => {
                        s.update(&state, time_now);
                        println!("Middle mouse : {:?}", self.middle.state.as_ref());
                    }
                    None => {
                        self.middle.state = Some(InputState::Pressed(time_now));
                        println!("Middle mouse : {:?}", self.middle.state.as_ref());
                    }
                }
            }
            ev::MouseButton::Right => {
                match &mut self.right.state {
                    Some(s) => {
                        s.update(&state, time_now);
                        println!("Right mouse : {:?}", self.right.state.as_ref());
                    }
                    None => {
                        self.right.state = Some(InputState::Pressed(time_now));
                        println!("Right mouse : {:?}", self.right.state.as_ref());
                    }
                }
            }
            _ => { /* Some other mouse button clicked. */ }
        }
    }
    pub fn forced_update(&mut self) {
        if let Some(InputState::Released(_,_)) = self.left.state   { self.left.state = None }
        if let Some(InputState::Released(_,_)) = self.middle.state { self.middle.state = None }
        if let Some(InputState::Released(_,_)) = self.right.state  { self.right.state = None }
    }
    pub fn get_left(self) -> Option<InputState> {
        self.left.state
    }
    pub fn get_middle(self) -> Option<InputState> {
        self.middle.state
    }
    pub fn get_right(self) -> Option<InputState> {
        self.right.state
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


    /// This should be called before the actual update to ensure the all events takes effect even
    /// winit doesn't produce any events..
    pub fn pre_update(&mut self) {
        // If mouse buttons were released previously, apply None to those states.
        self.mouse_buttons.forced_update();

        self.keyboard.retain(|_, state| match state { InputState::Released(_,_) => false, _ => true }); 

        // Update timer.
        let now = self.timer.elapsed().as_nanos();
        self.time_delta = now - self.time_now;
        self.time_now = now;
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
    /// Get the InputState of keyboard key.
    pub fn key_state(self, key: &Key) -> Option<InputState> {
        if let Some(val) = self.keyboard.get(key) {
            Some(val.clone())
        }
        else { None }
    }

    /// Get the InputState of mouse button.
    pub fn mouse_button_state(self, button: &ev::MouseButton) -> Option<InputState> {
        match button {
            ev::MouseButton::Left => { self.mouse_buttons.left.state } 
            ev::MouseButton::Middle => { self.mouse_buttons.middle.state } 
            ev::MouseButton::Right => { self.mouse_buttons.right.state } 
            _ => None
        }
    }
    /// Update the state of keyboard.
    fn track_keyboard(&mut self, evt: ev::KeyboardInput) {
        if let Some(key) = evt.virtual_keycode {
            match self.keyboard.get_mut(&key) {
                Some(state) => {
                    state.update(&evt.state, self.time_now) 
                }
                None => {
                    let _ = self.keyboard.insert(key, InputState::Pressed(self.time_now));
                }
            }
        }
    }
    /// Update the state of mouse buttons.
    fn track_mouse_button(&mut self, button: ev::MouseButton, state: ev::ElementState) {
        self.mouse_buttons.update(&button, &state, self.time_now);
    }
    /// Update the state of mouse wheel.
    fn track_mouse_wheel(&mut self, delta: ev::MouseScrollDelta) {
        println!("track_mouse_wheel");
    }
    /// Update the state of mouse movement.
    fn track_cursor_movement(&mut self, new_pos: PhysicalPosition<f64>) {
        match self.mouse_position.pos {
            None => { self.mouse_position.pos = Some(new_pos); }
            Some(old_position) => {
                self.mouse_delta = PhysicalPosition::<f64>::new(new_pos.x - old_position.x , new_pos.y - old_position.y);
                self.mouse_position.pos = Some(new_pos);
            }
        }
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
