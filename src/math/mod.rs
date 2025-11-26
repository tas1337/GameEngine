// Math library - everything written from scratch!

pub mod vec2;
pub mod vec3;
pub mod vec4;
pub mod mat4;
pub mod quat;

pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
pub use mat4::Mat4;
pub use quat::Quat;

// Common math constants and functions
pub const PI: f32 = 3.14159265359;
pub const TAU: f32 = 6.28318530718;
pub const DEG_TO_RAD: f32 = PI / 180.0;
pub const RAD_TO_DEG: f32 = 180.0 / PI;

#[inline]
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * DEG_TO_RAD
}

#[inline]
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * RAD_TO_DEG
}

#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

