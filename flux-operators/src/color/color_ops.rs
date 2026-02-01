//! Color operators: RgbaColor, HsvToRgb, RgbToHsv, BlendColors, SampleGradient,
//!                  AdjustBrightness, AdjustSaturation, ColorToVec4

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use crate::register_operators;
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::Color;

// ============================================================================
// RgbaColor Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "RgbaColor", category = "Color", description = "Create color from RGBA components")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct RgbaColorOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
    #[input(label = "R", default = 1.0)]
    r: f32,
    #[input(label = "G", default = 1.0)]
    g: f32,
    #[input(label = "B", default = 1.0)]
    b: f32,
    #[input(label = "A", default = 1.0)]
    a: f32,
    #[output(label = "Color")]
    color: Color,
}

impl RgbaColorOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let r = self.get_r(get_input);
        let g = self.get_g(get_input);
        let b = self.get_b(get_input);
        let a = self.get_a(get_input);
        self.set_color(Color::rgba(r, g, b, a));
    }
}

// ============================================================================
// HsvToRgb Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "HsvToRgb", category = "Color", description = "Convert HSV to RGB color")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct HsvToRgbOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
    #[input(label = "H", default = 0.0)]
    h: f32,
    #[input(label = "S", default = 1.0)]
    s: f32,
    #[input(label = "V", default = 1.0)]
    v: f32,
    #[input(label = "A", default = 1.0)]
    a: f32,
    #[output(label = "Color")]
    color: Color,
}

impl HsvToRgbOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let h = self.get_h(get_input);
        let s = self.get_s(get_input);
        let v = self.get_v(get_input);
        let a = self.get_a(get_input);
        let mut color = Color::from_hsv(h, s, v);
        color.a = a;
        self.set_color(color);
    }
}

// ============================================================================
// RgbToHsv Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "RgbToHsv", category = "Color", description = "Convert RGB color to HSV")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct RgbToHsvOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 3],
    #[input(label = "Color", default = [1.0, 1.0, 1.0, 1.0])]
    color: Color,
    #[output(label = "H")]
    h: f32,
    #[output(label = "S")]
    s: f32,
    #[output(label = "V")]
    v: f32,
}

impl RgbToHsvOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = self.get_color(get_input);
        let (h, s, v) = color.to_hsv();
        self.set_h(h);
        self.set_s(s);
        self.set_v(v);
    }
}

// ============================================================================
// BlendColors Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "BlendColors", category = "Color", description = "Blend two colors")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct BlendColorsOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0, 0.0, 1.0])]
    a: Color,
    #[input(label = "B", default = [1.0, 1.0, 1.0, 1.0])]
    b: Color,
    #[input(label = "T", default = 0.5)]
    t: f32,
    #[output(label = "Result")]
    result: Color,
}

impl BlendColorsOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        let t = self.get_t(get_input);
        self.set_result(Color::lerp(&a, &b, t));
    }
}

// ============================================================================
// SampleGradient Operator (NOT migrated - uses Gradient type)
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
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SampleGradient" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let gradient = self.inputs[0].resolve_gradient(get_input);
        let t = self.inputs[1].resolve_float(get_input);
        let color = gradient.sample(t);
        self.outputs[0].set_color(color.r, color.g, color.b, color.a);
    }
}

impl OperatorMeta for SampleGradientOp {
    fn category(&self) -> &'static str { "Color" }
    fn category_color(&self) -> [f32; 4] { [0.55, 0.35, 0.45, 1.0] }
    fn description(&self) -> &'static str { "Sample color from gradient at position" }
}

// ============================================================================
// AdjustBrightness Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "AdjustBrightness", category = "Color", description = "Adjust color brightness")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct AdjustBrightnessOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Color", default = [1.0, 1.0, 1.0, 1.0])]
    color: Color,
    #[input(label = "Amount", default = 0.0)]
    amount: f32,
    #[output(label = "Result")]
    result: Color,
}

impl AdjustBrightnessOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = self.get_color(get_input);
        let amount = self.get_amount(get_input);
        // Adjust brightness by modifying V in HSV
        let (h, s, v) = color.to_hsv();
        let new_v = (v + amount).clamp(0.0, 1.0);
        let mut result = Color::from_hsv(h, s, new_v);
        result.a = color.a;
        self.set_result(result);
    }
}

// ============================================================================
// AdjustSaturation Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "AdjustSaturation", category = "Color", description = "Adjust color saturation")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct AdjustSaturationOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Color", default = [1.0, 1.0, 1.0, 1.0])]
    color: Color,
    #[input(label = "Amount", default = 0.0)]
    amount: f32,
    #[output(label = "Result")]
    result: Color,
}

impl AdjustSaturationOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = self.get_color(get_input);
        let amount = self.get_amount(get_input);
        let (h, s, v) = color.to_hsv();
        let new_s = (s + amount).clamp(0.0, 1.0);
        let mut result = Color::from_hsv(h, new_s, v);
        result.a = color.a;
        self.set_result(result);
    }
}

// ============================================================================
// ColorToVec4 Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "ColorToVec4", category = "Color", description = "Convert color to Vec4")]
#[operator(category_color = [0.55, 0.35, 0.45, 1.0])]
#[allow(dead_code)]
pub struct ColorToVec4Op {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Color", default = [1.0, 1.0, 1.0, 1.0])]
    color: Color,
    #[output(label = "Vector")]
    vector: [f32; 4],
}

impl ColorToVec4Op {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let color = self.get_color(get_input);
        self.set_vector([color.r, color.g, color.b, color.a]);
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    // Register operators using #[derive(Operator)] in bulk
    register_operators!(
        registry,
        RgbaColorOp,
        HsvToRgbOp,
        RgbToHsvOp,
        BlendColorsOp,
        AdjustBrightnessOp,
        AdjustSaturationOp,
        ColorToVec4Op,
    );

    // SampleGradientOp uses Gradient type (not supported by derive macro)
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SampleGradient",
            category: "Color",
            description: "Sample color from gradient at position",
        },
        || capture_meta(SampleGradientOp::new()),
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
