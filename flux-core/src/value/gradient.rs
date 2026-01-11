//! Gradient type with color stops and sampling

use serde::{Deserialize, Serialize};

use super::Color;

/// A stop in a color gradient
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Position in the gradient (0.0 - 1.0)
    pub position: f32,
    /// Color at this position
    pub color: Color,
}

/// Color gradient with multiple stops
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    /// Gradient stops (should be sorted by position)
    pub stops: Vec<GradientStop>,
}

impl Gradient {
    /// Create a default black-to-white gradient
    pub fn new() -> Self {
        Self {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::BLACK,
                },
                GradientStop {
                    position: 1.0,
                    color: Color::WHITE,
                },
            ],
        }
    }

    /// Create a gradient between two colors
    pub fn two_color(start: Color, end: Color) -> Self {
        Self {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: start,
                },
                GradientStop {
                    position: 1.0,
                    color: end,
                },
            ],
        }
    }

    /// Add a stop to the gradient (maintains sorted order)
    pub fn add_stop(&mut self, position: f32, color: Color) {
        let stop = GradientStop {
            position: position.clamp(0.0, 1.0),
            color,
        };

        // Find insertion point to maintain sorted order
        let idx = self
            .stops
            .iter()
            .position(|s| s.position > stop.position)
            .unwrap_or(self.stops.len());

        self.stops.insert(idx, stop);
    }

    /// Sample the gradient at position t (0.0 - 1.0)
    pub fn sample(&self, t: f32) -> Color {
        if self.stops.is_empty() {
            return Color::BLACK;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        let t = t.clamp(0.0, 1.0);

        // Find surrounding stops
        let mut prev = &self.stops[0];
        for stop in &self.stops {
            if stop.position >= t {
                if stop.position == prev.position {
                    return stop.color;
                }
                let local_t = (t - prev.position) / (stop.position - prev.position);
                return Color::lerp(&prev.color, &stop.color, local_t);
            }
            prev = stop;
        }

        prev.color
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_sample() {
        let gradient = Gradient::two_color(Color::BLACK, Color::WHITE);

        let start = gradient.sample(0.0);
        assert_eq!(start.r, 0.0);

        let mid = gradient.sample(0.5);
        assert!((mid.r - 0.5).abs() < 0.01);

        let end = gradient.sample(1.0);
        assert_eq!(end.r, 1.0);
    }

    #[test]
    fn test_gradient_add_stop() {
        let mut gradient = Gradient::new();
        gradient.add_stop(0.5, Color::RED);

        assert_eq!(gradient.stops.len(), 3);
        assert_eq!(gradient.stops[1].position, 0.5);
    }
}
