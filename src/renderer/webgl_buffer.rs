// WebGL Buffer management - from scratch!

use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub struct GLBuffer {
    pub buffer: WebGlBuffer,
    pub target: u32,
}

impl GLBuffer {
    pub fn new(gl: &WebGl2RenderingContext, target: u32) -> Result<Self, JsValue> {
        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        Ok(Self { buffer, target })
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_buffer(self.target, Some(&self.buffer));
    }

    pub fn unbind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_buffer(self.target, None);
    }

    pub fn set_data(&self, gl: &WebGl2RenderingContext, data: &[f32], usage: u32) {
        self.bind(gl);
        unsafe {
            let view = js_sys::Float32Array::view(data);
            gl.buffer_data_with_array_buffer_view(self.target, &view, usage);
        }
    }

    pub fn set_data_u16(&self, gl: &WebGl2RenderingContext, data: &[u16], usage: u32) {
        self.bind(gl);
        unsafe {
            let view = js_sys::Uint16Array::view(data);
            gl.buffer_data_with_array_buffer_view(self.target, &view, usage);
        }
    }
}

/// Vertex with interleaved data
pub fn vertex_data_interleaved(vertices: &[crate::renderer::Vertex]) -> Vec<f32> {
    let mut data = Vec::with_capacity(vertices.len() * 12);
    for v in vertices {
        // Position (3)
        data.push(v.position[0]);
        data.push(v.position[1]);
        data.push(v.position[2]);
        // Normal (3)
        data.push(v.normal[0]);
        data.push(v.normal[1]);
        data.push(v.normal[2]);
        // UV (2)
        data.push(v.uv[0]);
        data.push(v.uv[1]);
        // Color (4)
        data.push(v.color[0]);
        data.push(v.color[1]);
        data.push(v.color[2]);
        data.push(v.color[3]);
    }
    data
}

