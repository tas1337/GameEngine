// Basic 2D texture helper for WebGL2

use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlTexture};

pub struct Texture {
    handle: WebGlTexture,
    width: i32,
    height: i32,
}

impl Texture {
    /// Create a texture from RGBA8 pixel data.
    pub fn from_rgba8(gl: &GL, width: u32, height: u32, data: &[u8]) -> Result<Self, JsValue> {
        let texture = gl
            .create_texture()
            .ok_or_else(|| JsValue::from_str("Failed to create texture"))?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

        // Ensure tightly packed rows
        gl.pixel_storei(GL::UNPACK_ALIGNMENT, 1);

        let width_i32 = width as i32;
        let height_i32 = height as i32;

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            width_i32,
            height_i32,
            0,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            Some(data),
        )?;

        // Use smooth sampling + mipmaps to avoid pixelation
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32);
        gl.tex_parameteri(
            GL::TEXTURE_2D,
            GL::TEXTURE_MIN_FILTER,
            GL::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        gl.generate_mipmap(GL::TEXTURE_2D);
        gl.bind_texture(GL::TEXTURE_2D, None);

        Ok(Self {
            handle: texture,
            width: width_i32,
            height: height_i32,
        })
    }

    /// Create a 1x1 solid-color texture (useful as a neutral default).
    pub fn solid_color(gl: &GL, color: [u8; 4]) -> Result<Self, JsValue> {
        Self::from_rgba8(gl, 1, 1, &color)
    }

    /// Bind the texture to the specified texture unit.
    pub fn bind(&self, gl: &GL, unit: u32) {
        gl.active_texture(GL::TEXTURE0 + unit);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.handle));
    }

    #[allow(dead_code)]
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

