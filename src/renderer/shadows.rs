// Shadow mapping system - built from scratch!
// Renders scene from light's perspective to create shadow map

use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlFramebuffer, WebGlTexture, WebGlProgram};
use crate::math::{Vec3, Mat4};

/// Shadow map resolution (power of 2 for best performance)
pub const SHADOW_MAP_SIZE: i32 = 2048;

/// Shadow mapping system
pub struct ShadowSystem {
    /// Framebuffer for shadow map rendering
    pub framebuffer: WebGlFramebuffer,
    /// Depth texture (the actual shadow map)
    pub depth_texture: WebGlTexture,
    /// Shader for rendering depth only
    pub depth_shader: WebGlProgram,
    /// Light space matrix (view * projection from sun)
    pub light_space_matrix: Mat4,
    /// Shadow map size
    pub size: i32,
}

impl ShadowSystem {
    pub fn new(gl: &GL) -> Result<Self, JsValue> {
        let size = SHADOW_MAP_SIZE;
        
        // Create depth texture for shadow map
        let depth_texture = gl.create_texture()
            .ok_or("Failed to create shadow map texture")?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&depth_texture));
        
        // Allocate depth texture
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,
            GL::DEPTH_COMPONENT32F as i32,
            size,
            size,
            0,
            GL::DEPTH_COMPONENT,
            GL::FLOAT,
            None,
        )?;
        
        // Shadow map texture parameters - NO compare mode, we do it manually in shader
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        
        // Create framebuffer
        let framebuffer = gl.create_framebuffer()
            .ok_or("Failed to create shadow framebuffer")?;
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&framebuffer));
        
        // Attach depth texture to framebuffer
        gl.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::DEPTH_ATTACHMENT,
            GL::TEXTURE_2D,
            Some(&depth_texture),
            0,
        );
        
        // We don't need color output for shadow map
        gl.draw_buffers(&js_sys::Array::new());
        gl.read_buffer(GL::NONE);
        
        // Check framebuffer completeness
        let status = gl.check_framebuffer_status(GL::FRAMEBUFFER);
        if status != GL::FRAMEBUFFER_COMPLETE {
            return Err(JsValue::from_str(&format!("Shadow framebuffer incomplete: {}", status)));
        }
        
        // Unbind framebuffer
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        
        // Create depth-only shader
        let depth_shader = create_depth_shader(gl)?;
        
        Ok(Self {
            framebuffer,
            depth_texture,
            depth_shader,
            light_space_matrix: Mat4::IDENTITY,
            size,
        })
    }
    
    /// Update light space matrix based on sun position
    pub fn update_light_matrix(&mut self, sun_direction: Vec3, scene_center: Vec3, scene_radius: f32) {
        // Light position (far away in opposite direction of sun)
        let light_pos = scene_center - sun_direction * scene_radius * 2.0;
        
        // Light view matrix (looking at scene center)
        let light_view = Mat4::look_at(&light_pos, &scene_center, &Vec3::Y);
        
        // Orthographic projection for directional light (sun)
        // Size based on scene radius to cover everything
        let ortho_size = scene_radius * 1.5;
        let light_proj = Mat4::orthographic(
            -ortho_size, ortho_size,  // left, right
            -ortho_size, ortho_size,  // bottom, top
            0.1, scene_radius * 4.0,  // near, far
        );
        
        self.light_space_matrix = light_proj.mul(&light_view);
    }
    
    /// Begin shadow map render pass
    pub fn begin_shadow_pass(&self, gl: &GL) {
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.framebuffer));
        gl.viewport(0, 0, self.size, self.size);
        gl.clear(GL::DEPTH_BUFFER_BIT);
        
        // Use depth shader
        gl.use_program(Some(&self.depth_shader));
        
        // Cull front faces to reduce shadow acne
        gl.cull_face(GL::FRONT);
    }
    
    /// End shadow map render pass
    pub fn end_shadow_pass(&self, gl: &GL, screen_width: i32, screen_height: i32) {
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        gl.viewport(0, 0, screen_width, screen_height);
        
        // Restore back face culling
        gl.cull_face(GL::BACK);
    }
    
    /// Bind shadow map texture for main render pass
    pub fn bind_shadow_map(&self, gl: &GL, texture_unit: u32) {
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.depth_texture));
    }
    
    /// Get uniform location helper
    pub fn get_depth_uniform(&self, gl: &GL, name: &str) -> Option<web_sys::WebGlUniformLocation> {
        gl.get_uniform_location(&self.depth_shader, name)
    }
}

/// Create depth-only shader for shadow map pass
fn create_depth_shader(gl: &GL) -> Result<WebGlProgram, JsValue> {
    let vert_source = r#"#version 300 es
        precision highp float;
        
        layout(location = 0) in vec3 a_position;
        
        // Per-instance model matrix
        layout(location = 4) in mat4 a_instanceModel;
        
        uniform mat4 u_lightSpaceMatrix;
        
        void main() {
            vec4 worldPos = a_instanceModel * vec4(a_position, 1.0);
            gl_Position = u_lightSpaceMatrix * worldPos;
        }
    "#;
    
    let frag_source = r#"#version 300 es
        precision highp float;
        
        void main() {
            // Depth is written automatically
            // We can optionally write to gl_FragDepth for custom depth
        }
    "#;
    
    compile_shader_program(gl, vert_source, frag_source)
}

/// Compile a shader program from source
fn compile_shader_program(gl: &GL, vert_src: &str, frag_src: &str) -> Result<WebGlProgram, JsValue> {
    let vert_shader = compile_shader(gl, GL::VERTEX_SHADER, vert_src)?;
    let frag_shader = compile_shader(gl, GL::FRAGMENT_SHADER, frag_src)?;
    
    let program = gl.create_program().ok_or("Failed to create program")?;
    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);
    
    if !gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap_or(false) {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        return Err(JsValue::from_str(&format!("Shader link error: {}", log)));
    }
    
    // Clean up individual shaders
    gl.delete_shader(Some(&vert_shader));
    gl.delete_shader(Some(&frag_shader));
    
    Ok(program)
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<web_sys::WebGlShader, JsValue> {
    let shader = gl.create_shader(shader_type).ok_or("Failed to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    
    if !gl.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap_or(false) {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        return Err(JsValue::from_str(&format!("Shader compile error: {}", log)));
    }
    
    Ok(shader)
}

