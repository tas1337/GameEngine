// Instanced rendering - draw millions of objects in one call!

use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext as GL;
use crate::math::{Mat4, Vec3};
use crate::ecs::Color;

/// Instance data for a single particle
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    pub model_matrix: [f32; 16],  // 4x4 matrix
    pub color: [f32; 4],           // RGBA
}

impl InstanceData {
    pub fn new(position: Vec3, scale: Vec3, color: Color) -> Self {
        let model = Mat4::translate(position.x, position.y, position.z)
            .mul(&Mat4::scale(scale.x, scale.y, scale.z));
        
        Self {
            model_matrix: model.as_array(),
            color: color.as_array(),
        }
    }

    pub fn as_bytes(data: &[InstanceData]) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<InstanceData>(),
            )
        }
    }
}

/// Instance buffer for dynamic instance data
pub struct InstanceBuffer {
    pub buffer: super::GLBuffer,
    pub capacity: usize,
}

impl InstanceBuffer {
    pub fn new(gl: &web_sys::WebGl2RenderingContext, capacity: usize) -> Result<Self, JsValue> {
        let buffer = super::GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        
        // Pre-allocate buffer with capacity
        let size = capacity * std::mem::size_of::<InstanceData>();
        buffer.bind(gl);
        gl.buffer_data_with_i32(GL::ARRAY_BUFFER, size as i32, GL::DYNAMIC_DRAW);
        
        Ok(Self { buffer, capacity })
    }

    pub fn update(&self, gl: &web_sys::WebGl2RenderingContext, instances: &[InstanceData]) {
        self.buffer.bind(gl);
        
        let bytes = InstanceData::as_bytes(instances);
        unsafe {
            let view = js_sys::Uint8Array::view(bytes);
            gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &view);
        }
    }

    pub fn setup_attributes(&self, gl: &web_sys::WebGl2RenderingContext) {
        self.buffer.bind(gl);
        
        let stride = std::mem::size_of::<InstanceData>() as i32;
        
        // Model matrix (4 vec4s, locations 4-7)
        for i in 0..4 {
            let location = 4 + i;
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_pointer_with_i32(
                location,
                4,                    // 4 floats per row
                GL::FLOAT,
                false,
                stride,
                (i * 16) as i32,     // Offset for each row (4 floats * 4 bytes)
            );
            gl.vertex_attrib_divisor(location, 1); // Instanced!
        }
        
        // Color (location 8)
        gl.enable_vertex_attrib_array(8);
        gl.vertex_attrib_pointer_with_i32(
            8,
            4,
            GL::FLOAT,
            false,
            stride,
            64, // Offset after model matrix (16 floats * 4 bytes)
        );
        gl.vertex_attrib_divisor(8, 1); // Instanced!
    }
}

