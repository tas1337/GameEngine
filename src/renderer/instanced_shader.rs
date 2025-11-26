// Instanced rendering shaders with sun lighting

pub const INSTANCED_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

// Per-vertex attributes
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in vec4 a_color;

// Per-instance attributes (instanced rendering!)
layout(location = 4) in mat4 a_instanceModel;  // locations 4,5,6,7
layout(location = 8) in vec4 a_instanceColor;

uniform mat4 u_viewProj;

out vec3 v_normal;
out vec2 v_uv;
out vec4 v_color;
out vec3 v_position;

void main() {
    // Use instance model matrix instead of uniform
    vec4 worldPos = a_instanceModel * vec4(a_position, 1.0);
    gl_Position = u_viewProj * worldPos;
    
    v_normal = mat3(a_instanceModel) * a_normal;
    v_uv = a_uv;
    v_color = a_instanceColor;  // Use instance color
    v_position = worldPos.xyz;
}
"#;

pub const INSTANCED_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 v_normal;
in vec2 v_uv;
in vec4 v_color;
in vec3 v_position;

out vec4 fragColor;

uniform vec3 u_sunDirection;
uniform vec3 u_sunColor;
uniform float u_ambientStrength;

void main() {
    vec3 normal = normalize(v_normal);
    
    // Diffuse lighting from sun
    float diffuse = max(dot(normal, u_sunDirection), 0.0);
    
    // Ambient light (darker at night)
    vec3 ambient = u_ambientStrength * vec3(1.0);
    
    // Combine lighting
    vec3 lighting = ambient + diffuse * u_sunColor;
    
    // Apply to object color
    vec3 finalColor = v_color.rgb * lighting;
    
    fragColor = vec4(finalColor, v_color.a);
}
"#;

