// WebGL context - raw 3D graphics like Three.js uses!

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

pub struct WebGLContext {
    pub gl: WebGl2RenderingContext,
    pub canvas: HtmlCanvasElement,
    pub width: u32,
    pub height: u32,
}

impl WebGLContext {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context("webgl2")?
            .ok_or("Failed to get WebGL2 context")?
            .dyn_into::<WebGl2RenderingContext>()?;

        let width = canvas.width();
        let height = canvas.height();

        // Enable depth testing for 3D
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_func(WebGl2RenderingContext::LEQUAL);

        // Enable backface culling
        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        Ok(Self {
            gl,
            canvas,
            width,
            height,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.gl.viewport(0, 0, width as i32, height as i32);
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT
        );
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        if self.height > 0 {
            self.width as f32 / self.height as f32
        } else {
            16.0 / 9.0
        }
    }
    
    pub fn get_viewport_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

