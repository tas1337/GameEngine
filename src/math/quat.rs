use super::vec3::Vec3;
use super::mat4::Mat4;

/// Quaternion for rotations (x, y, z, w)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Create from axis and angle
    #[inline]
    pub fn from_axis_angle(axis: &Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();
        let c = half_angle.cos();
        
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: c,
        }
    }

    /// Create from Euler angles (pitch, yaw, roll)
    #[inline]
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();

        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            *self
        }
    }

    /// Quaternion multiplication
    #[inline]
    pub fn mul(&self, other: &Quat) -> Quat {
        Quat {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    /// Convert to rotation matrix
    #[inline]
    pub fn to_mat4(&self) -> Mat4 {
        let q = self.normalize();
        
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        Mat4::new([
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0],
            [2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0],
            [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Spherical linear interpolation
    #[inline]
    pub fn slerp(&self, other: &Quat, t: f32) -> Quat {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
        
        let mut other = *other;
        if dot < 0.0 {
            other.x = -other.x;
            other.y = -other.y;
            other.z = -other.z;
            other.w = -other.w;
            dot = -dot;
        }

        if dot > 0.9995 {
            // Linear interpolation for very close quaternions
            return Quat {
                x: self.x + (other.x - self.x) * t,
                y: self.y + (other.y - self.y) * t,
                z: self.z + (other.z - self.z) * t,
                w: self.w + (other.w - self.w) * t,
            }.normalize();
        }

        let theta = dot.acos();
        let theta_t = theta * t;
        let sin_theta = theta.sin();
        let sin_theta_t = theta_t.sin();

        let s0 = (theta - theta_t).sin() / sin_theta;
        let s1 = sin_theta_t / sin_theta;

        Quat {
            x: self.x * s0 + other.x * s1,
            y: self.y * s0 + other.y * s1,
            z: self.z * s0 + other.z * s1,
            w: self.w * s0 + other.w * s1,
        }
    }
}

