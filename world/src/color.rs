use std::ops::{Mul, MulAssign};
use serde::{Deserialize, Serialize};

use crate::math::lerp;

/// RGBA color with float channels, matching Godot's `Color`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a 6- or 8-character hex string into a Color, matching Godot's `new Color("hex")`.
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        Self { r, g, b, a }
    }

    /// Linear interpolation between `self` and `to` on all 4 channels, matching Godot's `Color.Lerp`.
    #[inline]
    pub fn lerp(self, to: Color, weight: f32) -> Self {
        Self {
            r: lerp(self.r, to.r, weight),
            g: lerp(self.g, to.g, weight),
            b: lerp(self.b, to.b, weight),
            a: lerp(self.a, to.a, weight),
        }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Matches Godot's `operator*` on `Color`, multiplying all 4 channels including alpha.
impl Mul<f32> for Color {
    type Output = Color;
    #[inline]
    fn mul(self, rhs: f32) -> Color {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl MulAssign<f32> for Color {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}
