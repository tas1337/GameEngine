// Keyboard input - built from scratch!

use std::collections::HashSet;

#[derive(Clone)]
pub struct Keyboard {
    pressed: HashSet<String>,
    just_pressed: HashSet<String>,
    just_released: HashSet<String>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    pub fn press(&mut self, key: String) {
        if !self.pressed.contains(&key) {
            self.just_pressed.insert(key.clone());
        }
        self.pressed.insert(key);
    }

    pub fn release(&mut self, key: String) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub fn is_pressed(&self, key: &str) -> bool {
        self.pressed.contains(key)
    }

    pub fn is_just_pressed(&self, key: &str) -> bool {
        self.just_pressed.contains(key)
    }

    pub fn is_just_released(&self, key: &str) -> bool {
        self.just_released.contains(key)
    }

    /// Call this at the end of each frame to clear just_pressed/just_released
    pub fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

// Common key codes
pub const KEY_W: &str = "KeyW";
pub const KEY_A: &str = "KeyA";
pub const KEY_S: &str = "KeyS";
pub const KEY_D: &str = "KeyD";
pub const KEY_E: &str = "KeyE";
pub const KEY_SPACE: &str = "Space";
pub const KEY_SHIFT: &str = "ShiftLeft";
pub const KEY_CTRL: &str = "ControlLeft";
pub const ARROW_UP: &str = "ArrowUp";
pub const ARROW_DOWN: &str = "ArrowDown";
pub const ARROW_LEFT: &str = "ArrowLeft";
pub const ARROW_RIGHT: &str = "ArrowRight";

