// Mouse input - built from scratch!

use crate::math::Vec2;

#[derive(Clone, Copy)]
pub struct Mouse {
    pub position: Vec2,
    pub delta: Vec2,
    pub wheel_delta: f32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub left_just_pressed: bool,
    pub right_just_pressed: bool,
    pub left_just_released: bool,
    pub right_just_released: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            wheel_delta: 0.0,
            left_button: false,
            right_button: false,
            middle_button: false,
            left_just_pressed: false,
            right_just_pressed: false,
            left_just_released: false,
            right_just_released: false,
        }
    }

    pub fn update_position(&mut self, x: f32, y: f32) {
        let new_pos = Vec2::new(x, y);
        self.delta = new_pos - self.position;
        self.position = new_pos;
    }

    pub fn press_left(&mut self) {
        if !self.left_button {
            self.left_just_pressed = true;
        }
        self.left_button = true;
    }

    pub fn release_left(&mut self) {
        self.left_button = false;
        self.left_just_released = true;
    }

    pub fn press_right(&mut self) {
        if !self.right_button {
            self.right_just_pressed = true;
        }
        self.right_button = true;
    }

    pub fn release_right(&mut self) {
        self.right_button = false;
        self.right_just_released = true;
    }

    pub fn update_wheel(&mut self, delta: f32) {
        self.wheel_delta = delta;
    }

    /// Call at end of frame
    pub fn clear_frame_state(&mut self) {
        self.delta = Vec2::ZERO;
        self.wheel_delta = 0.0;
        self.left_just_pressed = false;
        self.right_just_pressed = false;
        self.left_just_released = false;
        self.right_just_released = false;
    }
}

