//! Color list operators
//!
//! Type-specific operators for ColorList:
//! - ColorListOp: Create ColorList from multi-input
//! - ColorListSample: Sample color at position (0-1)
//! - ColorListBlend: Blend all colors together

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::value::Color;
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_color_list(input: &InputPort, get_input: InputResolver) -> Vec<Color> {
    match input.connection {
        Some((node_id, output_idx)) => {
            let value = get_input(node_id, output_idx);
            match value {
                Value::ColorList(list) => list.to_vec(),
                Value::Color(c) => vec![c],
                Value::Vec4List(vl) => vl.iter().map(|v| Color::from_array(*v)).collect(),
                Value::Vec4(v) => vec![Color::from_array(v)],
                _ => Vec::new(),
            }
        }
        None => match &input.default {
            Value::ColorList(list) => list.to_vec(),
            Value::Color(c) => vec![*c],
            _ => Vec::new(),
        },
    }
}

fn collect_colors(input: &InputPort, get_input: InputResolver) -> Vec<Color> {
    if !input.connections.is_empty() {
        input
            .connections
            .iter()
            .map(|(node_id, output_idx)| {
                get_input(*node_id, *output_idx).as_color().unwrap_or(Color::BLACK)
            })
            .collect()
    } else {
        match &input.default {
            Value::ColorList(list) => list.to_vec(),
            Value::Color(c) => vec![*c],
            _ => Vec::new(),
        }
    }
}

// ============================================================================
// ColorList Operator (Creation)
// ============================================================================

pub struct ColorListOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl ColorListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::color_multi("Colors")],
            outputs: [OutputPort::color_list("List")],
        }
    }
}

impl Default for ColorListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ColorListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ColorList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let values = collect_colors(&self.inputs[0], get_input);
        self.outputs[0].value = Value::color_list(values);
    }
}

impl OperatorMeta for ColorListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Create a color list (palette) from colors" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Colors")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ColorListSample Operator
// ============================================================================

pub struct ColorListSampleOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl ColorListSampleOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::color_list("List"),
                InputPort::float("Position", 0.0),
            ],
            outputs: [OutputPort::color("Color")],
        }
    }
}

impl Default for ColorListSampleOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ColorListSampleOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ColorListSample" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_color_list(&self.inputs[0], get_input);
        let position = get_float(&self.inputs[1], get_input).clamp(0.0, 1.0);

        if list.is_empty() {
            self.outputs[0].set_color(0.0, 0.0, 0.0, 1.0);
            return;
        }

        if list.len() == 1 {
            let c = list[0];
            self.outputs[0].set_color(c.r, c.g, c.b, c.a);
            return;
        }

        // Interpolate between colors in the list
        let scaled = position * (list.len() - 1) as f32;
        let index = scaled.floor() as usize;
        let frac = scaled.fract();

        let color = if index >= list.len() - 1 {
            list[list.len() - 1]
        } else {
            let c1 = list[index];
            let c2 = list[index + 1];
            Color::rgba(
                c1.r + (c2.r - c1.r) * frac,
                c1.g + (c2.g - c1.g) * frac,
                c1.b + (c2.b - c1.b) * frac,
                c1.a + (c2.a - c1.a) * frac,
            )
        };

        self.outputs[0].set_color(color.r, color.g, color.b, color.a);
    }
}

impl OperatorMeta for ColorListSampleOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Sample color from palette at position (0-1)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            1 => Some(PortMeta::new("Position").with_range(0.0, 1.0)),
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
// ColorListBlend Operator
// ============================================================================

pub struct ColorListBlendOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ColorListBlendOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::color_list("List")],
            outputs: [OutputPort::color("Blended")],
        }
    }
}

impl Default for ColorListBlendOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ColorListBlendOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ColorListBlend" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_color_list(&self.inputs[0], get_input);

        if list.is_empty() {
            self.outputs[0].set_color(0.0, 0.0, 0.0, 1.0);
            return;
        }

        // Average all colors
        let mut sum_r = 0.0f32;
        let mut sum_g = 0.0f32;
        let mut sum_b = 0.0f32;
        let mut sum_a = 0.0f32;

        for c in &list {
            sum_r += c.r;
            sum_g += c.g;
            sum_b += c.b;
            sum_a += c.a;
        }

        let n = list.len() as f32;
        self.outputs[0].set_color(sum_r / n, sum_g / n, sum_b / n, sum_a / n);
    }
}

impl OperatorMeta for ColorListBlendOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Blend all colors in list (average)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Blended").with_shape(PinShape::TriangleFilled)),
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
            name: "ColorList",
            category: "List",
            description: "Create color list (palette)",
        },
        || capture_meta(ColorListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ColorListSample",
            category: "List",
            description: "Sample color from palette",
        },
        || capture_meta(ColorListSampleOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ColorListBlend",
            category: "List",
            description: "Blend all colors",
        },
        || capture_meta(ColorListBlendOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Color(Color::BLACK)
    }

    #[test]
    fn test_color_list_sample_edge() {
        let mut op = ColorListSampleOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::color_list(vec![
            Color::rgba(1.0, 0.0, 0.0, 1.0), // Red
            Color::rgba(0.0, 0.0, 1.0, 1.0), // Blue
        ]);

        // Sample at 0 -> Red
        op.inputs[1].default = Value::Float(0.0);
        op.compute(&ctx, &no_connections);
        if let Some(c) = op.outputs[0].value.as_color() {
            assert!((c.r - 1.0).abs() < 0.01);
            assert!((c.g).abs() < 0.01);
            assert!((c.b).abs() < 0.01);
        }

        // Sample at 1 -> Blue
        op.inputs[1].default = Value::Float(1.0);
        op.compute(&ctx, &no_connections);
        if let Some(c) = op.outputs[0].value.as_color() {
            assert!((c.r).abs() < 0.01);
            assert!((c.g).abs() < 0.01);
            assert!((c.b - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_color_list_sample_mid() {
        let mut op = ColorListSampleOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::color_list(vec![
            Color::rgba(1.0, 0.0, 0.0, 1.0), // Red
            Color::rgba(0.0, 0.0, 1.0, 1.0), // Blue
        ]);

        // Sample at 0.5 -> Purple (0.5, 0, 0.5)
        op.inputs[1].default = Value::Float(0.5);
        op.compute(&ctx, &no_connections);
        if let Some(c) = op.outputs[0].value.as_color() {
            assert!((c.r - 0.5).abs() < 0.01);
            assert!((c.g).abs() < 0.01);
            assert!((c.b - 0.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_color_list_blend() {
        let mut op = ColorListBlendOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::color_list(vec![
            Color::rgba(1.0, 0.0, 0.0, 1.0),
            Color::rgba(0.0, 1.0, 0.0, 1.0),
            Color::rgba(0.0, 0.0, 1.0, 1.0),
        ]);
        op.compute(&ctx, &no_connections);

        if let Some(c) = op.outputs[0].value.as_color() {
            assert!((c.r - 0.333).abs() < 0.01);
            assert!((c.g - 0.333).abs() < 0.01);
            assert!((c.b - 0.333).abs() < 0.01);
        }
    }
}
