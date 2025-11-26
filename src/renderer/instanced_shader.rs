// Instanced rendering shaders with sun lighting and shadows

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
uniform mat4 u_lightSpaceMatrix;

out vec3 v_normal;
out vec2 v_uv;
out vec4 v_color;
out vec3 v_position;
out vec4 v_lightSpacePos;

void main() {
    // Use instance model matrix instead of uniform
    vec4 worldPos = a_instanceModel * vec4(a_position, 1.0);
    gl_Position = u_viewProj * worldPos;
    
    // Calculate position in light space for shadow mapping
    v_lightSpacePos = u_lightSpaceMatrix * worldPos;
    
    // Proper normal transformation for non-uniform scaling
    mat3 normalMatrix = mat3(a_instanceModel);
    v_normal = normalize(normalMatrix * a_normal);
    
    v_uv = a_uv;
    // Combine baked vertex color (textures/gradients) with per-instance tint
    v_color = a_color * a_instanceColor;
    v_position = worldPos.xyz;
}
"#;

pub const INSTANCED_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 v_normal;
in vec2 v_uv;
in vec4 v_color;
in vec3 v_position;
in vec4 v_lightSpacePos;

out vec4 fragColor;

uniform vec3 u_sunDirection;
uniform vec3 u_sunColor;
uniform float u_ambientStrength;
uniform sampler2D u_shadowMap;
uniform bool u_shadowsEnabled;
uniform sampler2D u_baseColorTex;

// Calculate shadow with PCF (Percentage Closer Filtering) for soft shadows
float calculateShadow(vec4 lightSpacePos, vec3 normal, vec3 lightDir) {
    // Perspective divide
    vec3 projCoords = lightSpacePos.xyz / lightSpacePos.w;
    
    // Transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
    
    // Check if outside shadow map
    if (projCoords.x < 0.0 || projCoords.x > 1.0 ||
        projCoords.y < 0.0 || projCoords.y > 1.0 ||
        projCoords.z > 1.0) {
        return 0.0; // Not in shadow
    }
    
    // Current depth from light's perspective
    float currentDepth = projCoords.z;
    
    // Adaptive bias so the ground can still show contact shadows
    float normalDotLight = max(dot(normal, lightDir), 0.0);
    float bias = max(0.0025 * (1.0 - normalDotLight), 0.0004);
    float flatReceiver = smoothstep(0.75, 0.95, abs(normal.y));
    bias *= mix(1.0, 0.25, flatReceiver);
    
    // PCF - sample surrounding texels for soft shadows
    float shadow = 0.0;
    vec2 texelSize = 1.0 / vec2(textureSize(u_shadowMap, 0));
    
    // 5x5 PCF for softer shadows
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            float pcfDepth = texture(u_shadowMap, projCoords.xy + offset).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow = shadow / 25.0; // Average of 25 samples for softer shadows
    
    return clamp(shadow, 0.0, 1.0);
}

void main() {
    vec3 normal = normalize(v_normal);
    
    // Diffuse lighting from sun
    float diffuse = max(dot(normal, u_sunDirection), 0.0);
    
    // Calculate shadow
    float shadow = 0.0;
    if (u_shadowsEnabled) {
        shadow = calculateShadow(v_lightSpacePos, normal, u_sunDirection);
    }
    
    // Ambient light (darker at night)
    vec3 ambient = u_ambientStrength * vec3(1.0);
    
    // Apply stronger shadow influence (also dims ambient slightly)
    float shadow_strength = clamp(shadow, 0.0, 1.0);
    float flatReceiver = smoothstep(0.75, 0.95, abs(normal.y));
    float receiver_shadow = clamp(mix(shadow_strength, shadow_strength * 1.35, flatReceiver), 0.0, 1.0);
    vec3 shaded_ambient = ambient * (1.0 - receiver_shadow * 0.65);
    vec3 direct = (1.0 - receiver_shadow) * diffuse * u_sunColor;
    vec3 lighting = shaded_ambient + direct;

    // Sample base color texture and modulate with vertex/instance color
    vec4 albedo = texture(u_baseColorTex, v_uv) * v_color;
    
    // Apply to object color
    vec3 finalColor = albedo.rgb * lighting;
    
    fragColor = vec4(finalColor, albedo.a);
}
"#;

