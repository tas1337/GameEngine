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
    v_color = a_instanceColor;
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
    
    // Larger bias for ground (horizontal surfaces) to avoid shadow acne
    float normalDotLight = dot(normal, lightDir);
    float bias = max(0.01 * (1.0 - normalDotLight), 0.003);
    
    // Extra bias for nearly horizontal surfaces (like ground)
    if (abs(normal.y) > 0.9) {
        bias = 0.005;  // Fixed bias for ground
    }
    
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
    shadow /= 25.0; // Average of 25 samples for softer shadows
    
    return shadow;
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
    
    // Combine lighting with shadow
    // Shadow only affects direct sunlight, not ambient
    // shadow * 0.85 = 85% darker in shadow for visible effect
    vec3 lighting = ambient + (1.0 - shadow * 0.85) * diffuse * u_sunColor;
    
    // Apply to object color
    vec3 finalColor = v_color.rgb * lighting;
    
    fragColor = vec4(finalColor, v_color.a);
}
"#;

