// Core engine loop - built from scratch!

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
// use web_sys::Performance;  // Not currently used

/// Engine state and main loop
pub struct Engine {
    pub delta_time: f32,
    pub total_time: f32,
    pub frame_count: u64,
    pub fps: f32,
    
    last_frame_time: f64,
    fps_update_time: f64,
    fps_frame_count: u32,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            last_frame_time: 0.0,
            fps_update_time: 0.0,
            fps_frame_count: 0,
        }
    }

    pub fn update_time(&mut self, current_time: f64) {
        if self.last_frame_time == 0.0 {
            self.last_frame_time = current_time;
        }

        // Calculate delta time in seconds
        let dt = (current_time - self.last_frame_time) / 1000.0;
        self.delta_time = dt as f32;
        self.total_time += self.delta_time;
        self.last_frame_time = current_time;
        self.frame_count += 1;

        // Update FPS counter every second
        self.fps_frame_count += 1;
        if current_time - self.fps_update_time >= 1000.0 {
            self.fps = self.fps_frame_count as f32;
            self.fps_frame_count = 0;
            self.fps_update_time = current_time;
        }
    }
}

/// Main game loop callback
pub trait GameLoop {
    fn update(&mut self, engine: &Engine);
    fn render(&mut self, engine: &Engine);
}

/// Start the game loop
pub fn run_game_loop<F>(mut callback: F)
where
    F: FnMut(f64) + 'static,
{
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let performance = web_sys::window()
        .expect("should have window")
        .performance()
        .expect("should have performance");

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let time = performance.now();
        callback(time);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("should have window")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

