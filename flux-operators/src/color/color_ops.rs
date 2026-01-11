//! Color operators: RgbaColor, HsvToRgb, RgbToHsv, BlendColors, SampleGradient,
//!                  AdjustBrightness, AdjustSaturation, ColorToVec4

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::{Color, Gradient};

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_color(input: &InputPort, get_input: InputResolver) -> Color {
    match input.connection {
        Some((node_id, output_idx)) => {
            get_input(node_id, output_idx)
                .as_color()
                .unwrap_or(Color::WHITE)
        }
        None => input.default.as_color().unwrap_or(Color::WHITE),
    }
}

fn get_gradient(input: &InputPort, get_input: InputResolver) -> Gradient {
    match input.connection {
        Some((node_id, output_idx)) => {
            get_input(node_id, output_idx)
                .as_gradient()
                .cloned()
                .unwrap_or_default()
        }
        None => input.default.as_gradient().cloned().unwrap_or_default(),
    }
}

// ============================================================================
// RgbaColor Operator
// ============================================================================

pub struct RgbaColorOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl RgbaColorOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("R", 1.0),
                InputPort::float("G", 1.0),
                InputPort::float("B", 1.0),
                InputPort::float("A", 1.0),
            ],
            outputs: [OutputPort::color("Color")],
        }
    }
}

impl Default for RgbaColorOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for RgbaColorOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "RgbaColor" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let r = get_float(&self.inputs[0], get_input);
        let g = get_float(&self.inputs[1], get_input);
        let b = get_float(&self.inputs[2], get_input);
        let a = get_float(&self.inputs[3], get_input);
        self.outputs[0].set_color(r, g, b, a);
    }
}

impl OperatorMeta for RgbaColorOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Create color from RGBA components" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("R").with_range(0.0, 1.0)),
            1 => Some(PortMeta::new("G").with_range(0.0, 1.0)),
            2 => Some(PortMeta::new("B").with_range(0.0, 1.0)),
            3 => Some(PortMeta::new("A").with_range(0.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// HsvToRgb Operator
// ============================================================================

pub struct HsvToRgbOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl HsvToRgbOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("H", 0.0),
                InputPort::float("S", 1.0),
                InputPort::float("V", 1.0),
                InputPort::float("A", 1.0),
            ],
            outputs: [OutputPort::color("Color")],
        }
    }
}

impl Default for HsvToRgbOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for HsvToRgbOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "HsvToRgb" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let h = get_float(&self.inputs[0], get_input);
        let s = get_float(&self.inputs[1], get_input);
        let v = get_float(&self.inputs[2], get_input);
        let a = get_float(&self.inputs[3], get_input);
        let mut color = Color::from_hsv(h, s, v);
        color.a = a;
        self.outputs[0].set_color(color.r, color.g, color.b, color.a);
    }
}

impl OperatorMeta for HsvToRgbOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Convert HSV to RGB color" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("H").with_range(0.0, 360.0).with_unit("deg")),
            1 => Some(PortMeta::new("S").with_range(0.0, 1.0)),
            2 => Some(PortMeta::new("V").with_range(0.0, 1.0)),
            3 => Some(PortMeta::new("A").with_range(0.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// RgbToHsv Operator
// ============================================================================

pub struct RgbToHsvOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 3],
}

impl RgbToHsvOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::color("Color", [1.0, 1.0, 1.0, 1.0])],
            outputs: [
                OutputPort::float("H"),
                OutputPort::float("S"),
                OutputPort::float("V"),
            ],
        }
    }
}

impl Default for RgbToHsvOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for RgbToHsvOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "RgbToHsv" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = get_color(&self.inputs[0], get_input);
        let (h, s, v) = color.to_hsv();
        self.outputs[0].set_float(h);
        self.outputs[1].set_float(s);
        self.outputs[2].set_float(v);
    }
}

impl OperatorMeta for RgbToHsvOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Convert RGB color to HSV" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("H").with_shape(PinShape::TriangleFilled)),
            1 => Some(PortMeta::new("S").with_shape(PinShape::TriangleFilled)),
            2 => Some(PortMeta::new("V").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// BlendColors Operator
// ============================================================================

pub struct BlendColorsOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl BlendColorsOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::color("A", [0.0, 0.0, 0.0, 1.0]),
                InputPort::color("B", [1.0, 1.0, 1.0, 1.0]),
                InputPort::float("T", 0.5),
            ],
            outputs: [OutputPort::color("Result")],
        }
    }
}

impl Default for BlendColorsOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for BlendColorsOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "BlendColors" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_color(&self.inputs[0], get_input);
        let b = get_color(&self.inputs[1], get_input);
        let t = get_float(&self.inputs[2], get_input);
        let result = Color::lerp(&a, &b, t);
        self.outputs[0].set_color(result.r, result.g, result.b, result.a);
    }
}

impl OperatorMeta for BlendColorsOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Blend two colors" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            2 => Some(PortMeta::new("T").with_range(0.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// SampleGradient Operator
// ============================================================================

pub struct SampleGradientOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl SampleGradientOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::gradient("Gradient"),
                InputPort::float("T", 0.5),
            ],
            outputs: [OutputPort::color("Color")],
        }
    }
}

impl Default for SampleGradientOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SampleGradientOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SampleGradient" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let gradient = get_gradient(&self.inputs[0], get_input);
        let t = get_float(&self.inputs[1], get_input);
        let color = gradient.sample(t);
        self.outputs[0].set_color(color.r, color.g, color.b, color.a);
    }
}

impl OperatorMeta for SampleGradientOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Sample color from gradient at position" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Gradient")),
            1 => Some(PortMeta::new("T").with_range(0.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// AdjustBrightness Operator
// ============================================================================

pub struct AdjustBrightnessOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl AdjustBrightnessOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::color("Color", [1.0, 1.0, 1.0, 1.0]),
                InputPort::float("Amount", 0.0),
            ],
            outputs: [OutputPort::color("Result")],
        }
    }
}

impl Default for AdjustBrightnessOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AdjustBrightnessOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "AdjustBrightness" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = get_color(&self.inputs[0], get_input);
        let amount = get_float(&self.inputs[1], get_input);
        // Adjust brightness by modifying V in HSV
        let (h, s, v) = color.to_hsv();
        let new_v = (v + amount).clamp(0.0, 1.0);
        let mut result = Color::from_hsv(h, s, new_v);
        result.a = color.a;
        self.outputs[0].set_color(result.r, result.g, result.b, result.a);
    }
}

impl OperatorMeta for AdjustBrightnessOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Adjust color brightness" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color")),
            1 => Some(PortMeta::new("Amount").with_range(-1.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// AdjustSaturation Operator
// ============================================================================

pub struct AdjustSaturationOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl AdjustSaturationOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::color("Color", [1.0, 1.0, 1.0, 1.0]),
                InputPort::float("Amount", 0.0),
            ],
            outputs: [OutputPort::color("Result")],
        }
    }
}

impl Default for AdjustSaturationOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AdjustSaturationOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "AdjustSaturation" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = get_color(&self.inputs[0], get_input);
        let amount = get_float(&self.inputs[1], get_input);
        let (h, s, v) = color.to_hsv();
        let new_s = (s + amount).clamp(0.0, 1.0);
        let mut result = Color::from_hsv(h, new_s, v);
        result.a = color.a;
        self.outputs[0].set_color(result.r, result.g, result.b, result.a);
    }
}

impl OperatorMeta for AdjustSaturationOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Adjust color saturation" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color")),
            1 => Some(PortMeta::new("Amount").with_range(-1.0, 1.0)),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ColorToVec4 Operator
// ============================================================================

pub struct ColorToVec4Op {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ColorToVec4Op {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::color("Color", [1.0, 1.0, 1.0, 1.0])],
            outputs: [OutputPort::vec4("Vector")],
        }
    }
}

impl Default for ColorToVec4Op {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ColorToVec4Op {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ColorToVec4" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = get_color(&self.inputs[0], get_input);
        self.outputs[0].set_vec4([color.r, color.g, color.b, color.a]);
    }
}

impl OperatorMeta for ColorToVec4Op {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { category_colors::COLORS }
    fn description(&self) -> &'static str { "Convert color to Vec4" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Color")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "RgbaColor",
            category: "Color",
            description: "Create color from RGBA components",
        },
        || capture_meta(RgbaColorOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "HsvToRgb",
            category: "Color",
            description: "Convert HSV to RGB color",
        },
        || capture_meta(HsvToRgbOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "RgbToHsv",
            category: "Color",
            description: "Convert RGB color to HSV",
        },
        || capture_meta(RgbToHsvOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "BlendColors",
            category: "Color",
            description: "Blend two colors",
        },
        || capture_meta(BlendColorsOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SampleGradient",
            category: "Color",
            description: "Sample color from gradient at position",
        },
        || capture_meta(SampleGradientOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "AdjustBrightness",
            category: "Color",
            description: "Adjust color brightness",
        },
        || capture_meta(AdjustBrightnessOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "AdjustSaturation",
            category: "Color",
            description: "Adjust color saturation",
        },
        || capture_meta(AdjustSaturationOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ColorToVec4",
            category: "Color",
            description: "Convert color to Vec4",
        },
        || capture_meta(ColorToVec4Op::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_rgba_color() {
        let mut op = RgbaColorOp::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(0.5);
        op.inputs[2].default = Value::Float(0.0);
        op.inputs[3].default = Value::Float(1.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let color = op.outputs[0].value.as_color().unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_blend_colors() {
        let mut op = BlendColorsOp::new();
        op.inputs[0].default = Value::Color(Color::BLACK);
        op.inputs[1].default = Value::Color(Color::WHITE);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let color = op.outputs[0].value.as_color().unwrap();
        assert!((color.r - 0.5).abs() < 0.001);
        assert!((color.g - 0.5).abs() < 0.001);
        assert!((color.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_hsv_roundtrip() {
        let original = Color::rgba(0.8, 0.4, 0.2, 1.0);

        // RGB to HSV
        let mut rgb_to_hsv = RgbToHsvOp::new();
        rgb_to_hsv.inputs[0].default = Value::Color(original);
        let ctx = EvalContext::new();
        rgb_to_hsv.compute(&ctx, &no_connections);

        let h = rgb_to_hsv.outputs[0].value.as_float().unwrap();
        let s = rgb_to_hsv.outputs[1].value.as_float().unwrap();
        let v = rgb_to_hsv.outputs[2].value.as_float().unwrap();

        // HSV to RGB
        let mut hsv_to_rgb = HsvToRgbOp::new();
        hsv_to_rgb.inputs[0].default = Value::Float(h);
        hsv_to_rgb.inputs[1].default = Value::Float(s);
        hsv_to_rgb.inputs[2].default = Value::Float(v);
        hsv_to_rgb.inputs[3].default = Value::Float(1.0);
        hsv_to_rgb.compute(&ctx, &no_connections);

        let result = hsv_to_rgb.outputs[0].value.as_color().unwrap();
        assert!((result.r - original.r).abs() < 0.01);
        assert!((result.g - original.g).abs() < 0.01);
        assert!((result.b - original.b).abs() < 0.01);
    }
}
