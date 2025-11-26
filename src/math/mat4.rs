use super::vec3::Vec3;
use super::vec4::Vec4;

/// 4x4 Matrix stored in column-major order (like OpenGL/WebGPU expects)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    // Stored as columns: m[col][row]
    pub data: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub const ZERO: Mat4 = Mat4 {
        data: [[0.0; 4]; 4],
    };

    #[inline]
    pub fn new(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }

    /// Create translation matrix
    #[inline]
    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Mat4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0],
            ],
        }
    }

    /// Create scale matrix
    #[inline]
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Mat4 {
            data: [
                [x, 0.0, 0.0, 0.0],
                [0.0, y, 0.0, 0.0],
                [0.0, 0.0, z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Rotate around X axis (pitch)
    #[inline]
    pub fn rotate_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, s, 0.0],
                [0.0, -s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Rotate around Y axis (yaw)
    #[inline]
    pub fn rotate_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4 {
            data: [
                [c, 0.0, -s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Rotate around Z axis (roll)
    #[inline]
    pub fn rotate_z(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4 {
            data: [
                [c, s, 0.0, 0.0],
                [-s, c, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Perspective projection matrix
    #[inline]
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let tan_half_fov = (fov_y / 2.0).tan();
        
        Mat4 {
            data: [
                [1.0 / (aspect * tan_half_fov), 0.0, 0.0, 0.0],
                [0.0, 1.0 / tan_half_fov, 0.0, 0.0],
                [0.0, 0.0, -(far + near) / (far - near), -1.0],
                [0.0, 0.0, -(2.0 * far * near) / (far - near), 0.0],
            ],
        }
    }

    /// Orthographic projection matrix
    #[inline]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Mat4 {
            data: [
                [2.0 / (right - left), 0.0, 0.0, 0.0],
                [0.0, 2.0 / (top - bottom), 0.0, 0.0],
                [0.0, 0.0, -2.0 / (far - near), 0.0],
                [-(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near) / (far - near), 1.0],
            ],
        }
    }

    /// Look-at view matrix
    #[inline]
    pub fn look_at(eye: &Vec3, target: &Vec3, up: &Vec3) -> Self {
        let f = (*target - *eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(&f);

        Mat4 {
            data: [
                [s.x, u.x, -f.x, 0.0],
                [s.y, u.y, -f.y, 0.0],
                [s.z, u.z, -f.z, 0.0],
                [-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }

    /// Matrix multiplication
    #[inline]
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4::ZERO;
        
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = 
                    self.data[0][j] * other.data[i][0] +
                    self.data[1][j] * other.data[i][1] +
                    self.data[2][j] * other.data[i][2] +
                    self.data[3][j] * other.data[i][3];
            }
        }
        
        result
    }

    /// Transform a Vec4
    #[inline]
    pub fn mul_vec4(&self, v: &Vec4) -> Vec4 {
        Vec4::new(
            self.data[0][0] * v.x + self.data[1][0] * v.y + self.data[2][0] * v.z + self.data[3][0] * v.w,
            self.data[0][1] * v.x + self.data[1][1] * v.y + self.data[2][1] * v.z + self.data[3][1] * v.w,
            self.data[0][2] * v.x + self.data[1][2] * v.y + self.data[2][2] * v.z + self.data[3][2] * v.w,
            self.data[0][3] * v.x + self.data[1][3] * v.y + self.data[2][3] * v.z + self.data[3][3] * v.w,
        )
    }

    /// Get as flat array for GPU upload
    #[inline]
    pub fn as_array(&self) -> [f32; 16] {
        unsafe { std::mem::transmute(*self) }
    }
}

