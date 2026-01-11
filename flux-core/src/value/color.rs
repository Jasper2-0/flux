//! Color type with HSV conversion and interpolation

use serde::{Deserialize, Serialize};
use std::fmt;

/// RGBA color with components in 0.0-1.0 range
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Create an RGB color with alpha = 1.0
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create an RGBA color
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create from HSV (hue 0-360, saturation 0-1, value 0-1)
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h % 360.0;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(r + m, g + m, b + m)
    }

    /// Convert to HSV (returns hue 0-360, saturation 0-1, value 0-1)
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let v = max;
        let s = if max == 0.0 { 0.0 } else { delta / max };

        let h = if delta == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        (h, s, v)
    }

    /// Convert to array [r, g, b, a]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Create from array [r, g, b, a]
    pub fn from_array(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }

    /// Linear interpolation between two colors
    pub fn lerp(a: &Color, b: &Color, t: f32) -> Self {
        Self {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }

    /// Clamp all components to 0.0-1.0 range
    pub fn clamp(&self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }

    /// Get luminance (perceived brightness)
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rgba({:.2}, {:.2}, {:.2}, {:.2})",
            self.r, self.g, self.b, self.a
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let c = Color::rgb(1.0, 0.5, 0.25);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.25);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_hsv_roundtrip() {
        let original = Color::rgb(0.8, 0.4, 0.2);
        let (h, s, v) = original.to_hsv();
        let converted = Color::from_hsv(h, s, v);

        assert!((original.r - converted.r).abs() < 0.01);
        assert!((original.g - converted.g).abs() < 0.01);
        assert!((original.b - converted.b).abs() < 0.01);
    }

    #[test]
    fn test_color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = Color::lerp(&a, &b, 0.5);

        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }
}
