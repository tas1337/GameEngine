// WebGL Shader system - GLSL shaders from scratch!

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

pub struct ShaderProgram {
    pub program: WebGlProgram,
}

impl ShaderProgram {
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<Self, String> {
        let vertex_shader = compile_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_src,
        )?;

        let fragment_shader = compile_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_src,
        )?;

        let program = link_program(gl, &vertex_shader, &fragment_shader)?;

        Ok(Self { program })
    }

    pub fn use_program(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
    }

    pub fn get_attrib_location(&self, gl: &WebGl2RenderingContext, name: &str) -> i32 {
        gl.get_attrib_location(&self.program, name)
    }

    pub fn get_uniform_location(
        &self,
        gl: &WebGl2RenderingContext,
        name: &str,
    ) -> Option<web_sys::WebGlUniformLocation> {
        gl.get_uniform_location(&self.program, name)
    }
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    gl: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader program"))?;

    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program")))
    }
}

// Default 3D vertex shader
pub const VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in vec4 a_color;

uniform mat4 u_viewProj;
uniform mat4 u_model;

out vec3 v_normal;
out vec2 v_uv;
out vec4 v_color;
out vec3 v_position;

void main() {
    vec4 worldPos = u_model * vec4(a_position, 1.0);
    gl_Position = u_viewProj * worldPos;
    
    v_normal = mat3(u_model) * a_normal;
    v_uv = a_uv;
    v_color = a_color;
    v_position = worldPos.xyz;
}
"#;

// Default 3D fragment shader
pub const FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 v_normal;
in vec2 v_uv;
in vec4 v_color;
in vec3 v_position;

out vec4 fragColor;

uniform float u_time;

void main() {
    // Simple directional lighting
    vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
    vec3 normal = normalize(v_normal);
    
    float diffuse = max(dot(normal, lightDir), 0.2);
    
    vec3 color = v_color.rgb * diffuse;
    fragColor = vec4(color, v_color.a);
}
"#;

