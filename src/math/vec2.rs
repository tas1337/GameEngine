use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    pub const Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            *self
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }

    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

// Operator overloading
impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

