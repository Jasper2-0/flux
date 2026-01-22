//! Demo 19: Auto-Conversion at Connect Time
//!
//! This example demonstrates:
//! - Automatic type conversion when connecting incompatible ports
//! - ConversionOp nodes auto-inserted by the graph
//! - ConversionInserted events for UI synchronization
//! - Type categories for understanding type relationships
//!
//! Run with: `cargo run --example 19_auto_conversion`

use flux_core::{EvalContext, Id, InputPort, Operator, OutputPort, TypeCategory, Value, ValueType};
use flux_graph::{ConversionOp, Graph, GraphEvent};

// =============================================================================
// Custom operators for demonstration
// =============================================================================

/// A simple operator that outputs a Float value
struct FloatSource {
    id: Id,
    outputs: Vec<OutputPort>,
    value: f32,
}

impl FloatSource {
    fn new(value: f32) -> Self {
        let mut output = OutputPort::float("Out");
        output.set(Value::Float(value));
        Self {
            id: Id::new(),
            outputs: vec![output],
            value,
        }
    }
}

impl Operator for FloatSource {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "FloatSource"
    }
    fn inputs(&self) -> &[InputPort] {
        &[]
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut []
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
        self.outputs[0].set(Value::Float(self.value));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// An operator that accepts Vec3 input and outputs it
struct Vec3Processor {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl Vec3Processor {
    fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::new("In", Value::Vec3([0.0, 0.0, 0.0]))],
            outputs: vec![OutputPort::vec3("Out")],
        }
    }
}

impl Operator for Vec3Processor {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Vec3Processor"
    }
    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
        let input = if let Some((node_id, output_idx)) = self.inputs[0].connection {
            get_input(node_id, output_idx)
        } else {
            self.inputs[0].default.clone()
        };
        self.outputs[0].set(input);
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 19: Auto-Conversion at Connect    ║");
    println!("╚════════════════════════════════════════╝\n");

    // =========================================================================
    // Part 1: Basic Auto-Conversion
    // =========================================================================
    println!("═══ Part 1: Basic Auto-Conversion ═══\n");

    let mut graph = Graph::new();

    // Create a Float source outputting 2.5
    let float_source = graph.add(FloatSource::new(2.5));
    println!("Created FloatSource (outputs Float 2.5)");

    // Create a Vec3 processor (expects Vec3 input)
    let vec3_processor_id = {
        let proc = Vec3Processor::new();
        let id = proc.id;
        graph.add(proc);
        id
    };
    println!("Created Vec3Processor (expects Vec3 input)");

    // Clear events from node additions
    graph.clear_events();

    // Connect Float -> Vec3 (types don't match!)
    println!("\nAttempting to connect Float -> Vec3...");
    let result = graph.connect(float_source, 0, vec3_processor_id, 0);

    match result {
        Ok(Some(conv_id)) => {
            println!("  Auto-inserted ConversionOp (id: {:?})", conv_id);

            // Check what was inserted
            let conv_op = graph.get(conv_id).unwrap();
            println!("  Conversion node name: {}", conv_op.name());

            // Downcast to ConversionOp to get more details
            if let Some(conv) = conv_op.as_any().downcast_ref::<ConversionOp>() {
                println!(
                    "  Converts: {:?} -> {:?}",
                    conv.source_type(),
                    conv.target_type()
                );
                println!("  Is synthetic: {}", conv.is_synthetic());
            }
        }
        Ok(None) => {
            println!("  Direct connection (no conversion needed)");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Check the events emitted
    println!("\nEvents emitted:");
    for event in graph.drain_events() {
        match event {
            GraphEvent::NodeAdded { id } => {
                println!("  NodeAdded: {:?}", id);
            }
            GraphEvent::Connected {
                source,
                target,
                source_output,
                target_input,
            } => {
                println!(
                    "  Connected: {:?}[{}] -> {:?}[{}]",
                    source, source_output, target, target_input
                );
            }
            GraphEvent::ConversionInserted {
                conversion_node,
                source_type,
                target_type,
            } => {
                println!(
                    "  ConversionInserted: {:?} ({:?} -> {:?})",
                    conversion_node, source_type, target_type
                );
            }
            _ => {}
        }
    }

    // =========================================================================
    // Part 2: Evaluation Through Conversion
    // =========================================================================
    println!("\n═══ Part 2: Evaluation Through Conversion ═══\n");

    let ctx = EvalContext::new();
    let result = graph.evaluate(vec3_processor_id, 0, &ctx).unwrap();

    println!("Input value: Float(2.5)");
    println!("Output value: {:?}", result);
    println!("  Float 2.5 was broadcast to Vec3 [2.5, 2.5, 2.5]");

    // =========================================================================
    // Part 3: Type Categories
    // =========================================================================
    println!("\n═══ Part 3: Type Categories ═══\n");

    println!("Type categories help group related types:");
    println!();

    let types = [
        ValueType::Float,
        ValueType::Int,
        ValueType::Vec2,
        ValueType::Vec3,
        ValueType::Vec4,
        ValueType::Color,
        ValueType::FloatList,
    ];

    for t in types {
        let cats = t.categories();
        let cat_names: Vec<_> = cats.iter().map(|c| format!("{:?}", c)).collect();
        println!("  {:14} -> [{}]", format!("{:?}", t), cat_names.join(", "));
    }

    println!();
    println!("Checking category membership:");
    println!(
        "  Float is Numeric: {}",
        ValueType::Float.is_in_category(TypeCategory::Numeric)
    );
    println!(
        "  Vec3 is Vector:   {}",
        ValueType::Vec3.is_in_category(TypeCategory::Vector)
    );
    println!(
        "  Vec4 is ColorLike:{}",
        ValueType::Vec4.is_in_category(TypeCategory::ColorLike)
    );
    println!(
        "  Color is Vector:  {}",
        ValueType::Color.is_in_category(TypeCategory::Vector)
    );

    // =========================================================================
    // Part 4: Incompatible Types
    // =========================================================================
    println!("\n═══ Part 4: Incompatible Types ═══\n");

    // String cannot be converted to Vec3
    struct StringSource {
        id: Id,
        outputs: Vec<OutputPort>,
    }
    impl StringSource {
        fn new() -> Self {
            let mut output = OutputPort::string("Out");
            output.set(Value::String("hello".into()));
            Self {
                id: Id::new(),
                outputs: vec![output],
            }
        }
    }
    impl Operator for StringSource {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "StringSource"
        }
        fn inputs(&self) -> &[InputPort] {
            &[]
        }
        fn inputs_mut(&mut self) -> &mut [InputPort] {
            &mut []
        }
        fn outputs(&self) -> &[OutputPort] {
            &self.outputs
        }
        fn outputs_mut(&mut self) -> &mut [OutputPort] {
            &mut self.outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {}
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    let string_source = graph.add(StringSource::new());
    let vec3_sink = graph.add(Vec3Processor::new());

    println!("Attempting to connect String -> Vec3...");
    let result = graph.connect(string_source, 0, vec3_sink, 0);
    match result {
        Ok(_) => println!("  Connected successfully"),
        Err(e) => println!("  Error: {}", e),
    }

    println!();
    println!("Graph stats: {:?}", graph.stats());
}
