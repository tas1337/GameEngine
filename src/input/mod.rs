// Input system - built from scratch!

pub mod keyboard;
pub mod mouse;

pub use keyboard::*;
pub use mouse::*;

#[derive(Clone)]
pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }

    pub fn clear_frame_state(&mut self) {
        self.keyboard.clear_frame_state();
        self.mouse.clear_frame_state();
    }
}

