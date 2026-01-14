//! Example 30: List Processing
//!
//! This example demonstrates Flux's list operators in a connected graph context:
//! - Multi-input collection (FloatListOp with multiple graph connections)
//! - Statistical analysis pipeline (Sum, Average, Min, Max)
//! - List transformations with connected parameters (ListMap)
//! - New operators: ListFilter, ListConcat, ListSlice
//!
//! Key insight: Lists enable batch processing where a single connection
//! carries multiple values through the dataflow graph.
//!
//! Run with: cargo run --example 30_list_processing

use flux_core::{EvalContext, Value};
use flux_graph::Graph;
use flux_operators::{
    AddOp, ConstantOp, SineWaveOp, SubtractOp,
    FloatListOp, ListAverageOp, ListGetOp, ListLengthOp,
    ListMapOp, ListMaxOp, ListMinOp, ListSumOp,
    ListFilterOp, ListConcatOp, ListSliceOp,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 30: List Processing               ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_multi_input_collection();
    demo_statistical_pipeline();
    demo_list_transformation();
    demo_filter_concat_slice();
    demo_sensor_data_pipeline();

    println!("\n=== Summary ===\n");
    println!("List operators enable batch processing in Flux:");
    println!();
    println!("  Creation:");
    println!("    FloatListOp: Create list from default or multiple connections");
    println!("    Value::FloatList: Direct list values");
    println!();
    println!("  Analysis:");
    println!("    ListSum, ListAverage: Aggregate statistics");
    println!("    ListMin, ListMax: Find extrema");
    println!("    ListLength: Count elements");
    println!();
    println!("  Transformation:");
    println!("    ListMap: Apply scale + offset to all elements");
    println!("    ListGet: Extract single element by index");
    println!("    ListFilter: Keep elements matching threshold condition");
    println!("    ListSlice: Extract subrange (supports negative indices)");
    println!("    ListConcat: Merge two lists");
    println!();
    println!("  Multi-Input Pattern:");
    println!("    osc1 ─┐");
    println!("    osc2 ─┼─▶ FloatListOp ─▶ [values...]");
    println!("    osc3 ─┘   (collects all)");
}

/// Part 1: Multi-input collection - FloatListOp collects from multiple graph nodes
fn demo_multi_input_collection() {
    println!("=== Part 1: Multi-Input Collection ===\n");

    println!("  FloatListOp can collect values from MULTIPLE graph connections.");
    println!("  Each call to graph.connect() APPENDS to the multi-input port.");
    println!();

    let mut graph = Graph::new();

    // Create three oscillators at different frequencies
    let osc1 = graph.add(SineWaveOp::new());
    let osc2 = graph.add(SineWaveOp::new());
    let osc3 = graph.add(SineWaveOp::new());

    // Configure frequencies: 1Hz, 2Hz, 3Hz
    graph.set_input_default(osc1, 0, Value::Float(1.0));
    graph.set_input_default(osc2, 0, Value::Float(2.0));
    graph.set_input_default(osc3, 0, Value::Float(3.0));

    // Create FloatListOp and connect ALL THREE oscillators to input 0
    // This works because FloatListOp uses InputPort::float_multi() which
    // sets is_multi_input = true. Each connect() call APPENDS!
    let list_op = graph.add(FloatListOp::new());
    graph.connect(osc1, 0, list_op, 0).unwrap();  // First connection
    graph.connect(osc2, 0, list_op, 0).unwrap();  // Appends (doesn't replace!)
    graph.connect(osc3, 0, list_op, 0).unwrap();  // Appends again

    // Now analyze the dynamic list
    let avg_op = graph.add(ListAverageOp::new());
    let sum_op = graph.add(ListSumOp::new());
    graph.connect(list_op, 0, avg_op, 0).unwrap();
    graph.connect(list_op, 0, sum_op, 0).unwrap();

    println!("  Graph structure:");
    println!("    SineWave(1Hz) ─┐");
    println!("    SineWave(2Hz) ─┼─▶ FloatListOp ─┬─▶ ListAverage ─▶ avg");
    println!("    SineWave(3Hz) ─┘   (3 inputs)   └─▶ ListSum ─────▶ sum");
    println!();

    // Show how the list changes over time
    println!("  {:>6}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
             "Time", "Osc1", "Osc2", "Osc3", "Average", "Sum");
    println!("  {:->6}  {:->8}  {:->8}  {:->8}  {:->8}  {:->8}", "", "", "", "", "", "");

    for frame in 0..6 {
        let mut ctx = EvalContext::new();
        ctx.time = frame as f64 * 0.1;

        let list_val = graph.evaluate(list_op, 0, &ctx).unwrap();
        let avg = graph.evaluate(avg_op, 0, &ctx).unwrap().as_float().unwrap();
        let sum = graph.evaluate(sum_op, 0, &ctx).unwrap().as_float().unwrap();

        if let Value::FloatList(values) = list_val {
            println!("  {:>6.2}  {:>+8.4}  {:>+8.4}  {:>+8.4}  {:>+8.4}  {:>+8.4}",
                     ctx.time, values[0], values[1], values[2], avg, sum);
        }
    }

    println!();
    println!("  Key insight: Multiple connect() calls to a multi-input port");
    println!("  create a dynamic list from live graph outputs!");
    println!();
}

/// Part 2: Statistical analysis pipeline
fn demo_statistical_pipeline() {
    println!("=== Part 2: Statistical Analysis Pipeline ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // Create a list source (using default value for simplicity)
    let list_source = graph.add(FloatListOp::new());
    graph.set_input_default(list_source, 0, Value::float_list(vec![
        15.0, 22.0, 8.0, 31.0, 19.0, 27.0, 12.0, 25.0
    ]));

    // Connect to multiple analysis operators (fan-out pattern)
    let sum_op = graph.add(ListSumOp::new());
    let avg_op = graph.add(ListAverageOp::new());
    let min_op = graph.add(ListMinOp::new());
    let max_op = graph.add(ListMaxOp::new());
    let len_op = graph.add(ListLengthOp::new());

    graph.connect(list_source, 0, sum_op, 0).unwrap();
    graph.connect(list_source, 0, avg_op, 0).unwrap();
    graph.connect(list_source, 0, min_op, 0).unwrap();
    graph.connect(list_source, 0, max_op, 0).unwrap();
    graph.connect(list_source, 0, len_op, 0).unwrap();

    // Compute range: max - min
    let range_op = graph.add(SubtractOp::new());
    graph.connect(max_op, 0, range_op, 0).unwrap();
    graph.connect(min_op, 0, range_op, 1).unwrap();

    println!("  Input: [15, 22, 8, 31, 19, 27, 12, 25]");
    println!();
    println!("  Graph structure:");
    println!("                    ┌─▶ ListSum ────▶ sum");
    println!("                    ├─▶ ListAverage ─▶ avg");
    println!("    FloatList ──────┼─▶ ListMin ─┬──▶ min");
    println!("                    ├─▶ ListMax ─┼──▶ max");
    println!("                    │            └─▶ Subtract ─▶ range");
    println!("                    └─▶ ListLength ─▶ count");
    println!();

    // Evaluate all outputs
    let sum = graph.evaluate(sum_op, 0, &ctx).unwrap().as_float().unwrap();
    let avg = graph.evaluate(avg_op, 0, &ctx).unwrap().as_float().unwrap();
    let min = graph.evaluate(min_op, 0, &ctx).unwrap().as_float().unwrap();
    let max = graph.evaluate(max_op, 0, &ctx).unwrap().as_float().unwrap();
    let range = graph.evaluate(range_op, 0, &ctx).unwrap().as_float().unwrap();
    let count = graph.evaluate(len_op, 0, &ctx).unwrap().as_int().unwrap();

    println!("  Results:");
    println!("    Count:   {}", count);
    println!("    Sum:     {:.1}", sum);
    println!("    Average: {:.2}", avg);
    println!("    Min:     {:.1}", min);
    println!("    Max:     {:.1}", max);
    println!("    Range:   {:.1} (max - min)", range);
    println!();
}

/// Part 3: List transformation with connected parameters
fn demo_list_transformation() {
    println!("=== Part 3: List Transformation (Dynamic Parameters) ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // Create a list of values
    let source_list = graph.add(FloatListOp::new());
    graph.set_input_default(source_list, 0, Value::float_list(vec![1.0, 2.0, 3.0, 4.0, 5.0]));

    // Create dynamic scale and offset from graph operators
    let scale = graph.add(ConstantOp::new(2.0));
    let offset = graph.add(ConstantOp::new(100.0));

    // ListMap applies: result[i] = list[i] * scale + offset
    let map_op = graph.add(ListMapOp::new());
    graph.connect(source_list, 0, map_op, 0).unwrap();  // List input
    graph.connect(scale, 0, map_op, 1).unwrap();        // Scale
    graph.connect(offset, 0, map_op, 2).unwrap();       // Offset

    println!("  Input list: [1, 2, 3, 4, 5]");
    println!("  Scale: 2.0, Offset: 100.0");
    println!("  Formula: result[i] = input[i] * 2 + 100");
    println!();

    let result = graph.evaluate(map_op, 0, &ctx).unwrap();

    if let Value::FloatList(mapped) = result {
        println!("  Output: {:?}", mapped);
    }

    // Chain another transformation
    let scale2 = graph.add(ConstantOp::new(0.5));
    let offset2 = graph.add(ConstantOp::new(-50.0));
    let map_op2 = graph.add(ListMapOp::new());

    graph.connect(map_op, 0, map_op2, 0).unwrap();   // Previous result
    graph.connect(scale2, 0, map_op2, 1).unwrap();
    graph.connect(offset2, 0, map_op2, 2).unwrap();

    println!();
    println!("  Chained transformation: (result * 0.5) - 50");

    let result2 = graph.evaluate(map_op2, 0, &ctx).unwrap();

    if let Value::FloatList(final_list) = result2 {
        println!("  Final output: {:?}", final_list);
        println!("  (Back to original values!)");
    }
    println!();
}

/// Part 4: Filter, Concat, and Slice operations
fn demo_filter_concat_slice() {
    println!("=== Part 4: Filter, Concat, and Slice ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // --- ListFilter Demo ---
    println!("  ListFilter: Keep elements matching a threshold condition");
    println!();

    let data = graph.add(FloatListOp::new());
    graph.set_input_default(data, 0, Value::float_list(vec![
        5.0, 12.0, 3.0, 18.0, 7.0, 25.0, 9.0, 15.0
    ]));

    // Filter: keep values > 10 (mode 0 = greater than)
    let filter_op = graph.add(ListFilterOp::new());
    graph.connect(data, 0, filter_op, 0).unwrap();
    graph.set_input_default(filter_op, 1, Value::Float(10.0));  // Threshold
    graph.set_input_default(filter_op, 2, Value::Int(0));       // Mode: 0=GT

    println!("  Input:     [5, 12, 3, 18, 7, 25, 9, 15]");
    println!("  Filter:    > 10");

    let filtered = graph.evaluate(filter_op, 0, &ctx).unwrap();
    if let Value::FloatList(vals) = filtered {
        println!("  Result:    {:?}", vals);
    }
    println!();

    // Filter: keep values <= 10 (mode 3 = less than or equal)
    let filter_op2 = graph.add(ListFilterOp::new());
    graph.connect(data, 0, filter_op2, 0).unwrap();
    graph.set_input_default(filter_op2, 1, Value::Float(10.0));
    graph.set_input_default(filter_op2, 2, Value::Int(3));  // Mode: 3=LTE

    println!("  Filter:    <= 10");
    let filtered2 = graph.evaluate(filter_op2, 0, &ctx).unwrap();
    if let Value::FloatList(vals) = filtered2 {
        println!("  Result:    {:?}", vals);
    }
    println!();

    // --- ListConcat Demo ---
    println!("  ListConcat: Merge two lists into one");
    println!();

    let list_a = graph.add(FloatListOp::new());
    let list_b = graph.add(FloatListOp::new());
    graph.set_input_default(list_a, 0, Value::float_list(vec![1.0, 2.0, 3.0]));
    graph.set_input_default(list_b, 0, Value::float_list(vec![4.0, 5.0, 6.0]));

    let concat_op = graph.add(ListConcatOp::new());
    graph.connect(list_a, 0, concat_op, 0).unwrap();
    graph.connect(list_b, 0, concat_op, 1).unwrap();

    println!("  List A:    [1, 2, 3]");
    println!("  List B:    [4, 5, 6]");

    let concatenated = graph.evaluate(concat_op, 0, &ctx).unwrap();
    if let Value::FloatList(vals) = concatenated {
        println!("  Concat:    {:?}", vals);
    }
    println!();

    // --- ListSlice Demo ---
    println!("  ListSlice: Extract a subrange (Python-style negative indices)");
    println!();

    let source = graph.add(FloatListOp::new());
    graph.set_input_default(source, 0, Value::float_list(vec![
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0
    ]));

    println!("  Source:    [10, 20, 30, 40, 50, 60, 70]");
    println!();

    // Slice [1:4] - indices 1, 2, 3
    let slice1 = graph.add(ListSliceOp::new());
    graph.connect(source, 0, slice1, 0).unwrap();
    graph.set_input_default(slice1, 1, Value::Int(1));  // Start
    graph.set_input_default(slice1, 2, Value::Int(4));  // End (exclusive)

    let result1 = graph.evaluate(slice1, 0, &ctx).unwrap();
    print!("  [1:4]  →   ");
    if let Value::FloatList(vals) = result1 {
        println!("{:?}", vals);
    }

    // Slice [:3] - first 3 elements
    let slice2 = graph.add(ListSliceOp::new());
    graph.connect(source, 0, slice2, 0).unwrap();
    graph.set_input_default(slice2, 1, Value::Int(0));
    graph.set_input_default(slice2, 2, Value::Int(3));

    let result2 = graph.evaluate(slice2, 0, &ctx).unwrap();
    print!("  [:3]   →   ");
    if let Value::FloatList(vals) = result2 {
        println!("{:?}", vals);
    }

    // Slice [-3:] - last 3 elements (negative start, default end)
    let slice3 = graph.add(ListSliceOp::new());
    graph.connect(source, 0, slice3, 0).unwrap();
    graph.set_input_default(slice3, 1, Value::Int(-3));
    // End defaults to i32::MAX which means "to end"

    let result3 = graph.evaluate(slice3, 0, &ctx).unwrap();
    print!("  [-3:]  →   ");
    if let Value::FloatList(vals) = result3 {
        println!("{:?}", vals);
    }

    // Slice [2:-2] - from index 2 to 2 before end
    let slice4 = graph.add(ListSliceOp::new());
    graph.connect(source, 0, slice4, 0).unwrap();
    graph.set_input_default(slice4, 1, Value::Int(2));
    graph.set_input_default(slice4, 2, Value::Int(-2));

    let result4 = graph.evaluate(slice4, 0, &ctx).unwrap();
    print!("  [2:-2] →   ");
    if let Value::FloatList(vals) = result4 {
        println!("{:?}", vals);
    }

    println!();
    println!("  Filter modes: 0=GT(>), 1=LT(<), 2=GTE(>=), 3=LTE(<=)");
    println!();
}

/// Part 5: Practical use case - dynamic list processing with time
fn demo_sensor_data_pipeline() {
    println!("=== Part 5: Dynamic List Processing ===\n");

    println!("  Scenario: Analyzing sensor readings that change over time");
    println!("  - Raw voltage readings: [1.2V, 2.5V, 1.8V, 3.1V, 2.2V]");
    println!("  - Scale controlled by oscillating multiplier");
    println!("  - Demonstrates lists + time-based parameters");
    println!();

    let mut graph = Graph::new();

    // Static list of sensor readings (voltages)
    let readings = graph.add(FloatListOp::new());
    graph.set_input_default(readings, 0, Value::float_list(vec![1.2, 2.5, 1.8, 3.1, 2.2]));

    // Time-varying scale factor using SineWave
    // Simulates a gain control that oscillates between 15 and 25
    let oscillator = graph.add(SineWaveOp::new());
    graph.set_input_default(oscillator, 0, Value::Float(0.5));  // Frequency
    graph.set_input_default(oscillator, 1, Value::Float(5.0));  // Amplitude (±5)
    graph.set_input_default(oscillator, 2, Value::Float(0.0));  // Phase

    // Add base scale: 20 + oscillator output = range [15, 25]
    let base_scale = graph.add(ConstantOp::new(20.0));
    let dynamic_scale = graph.add(AddOp::new());
    graph.connect(base_scale, 0, dynamic_scale, 0).unwrap();
    graph.connect(oscillator, 0, dynamic_scale, 1).unwrap();

    // Convert using dynamic scale: voltage * scale = temperature
    let zero_offset = graph.add(ConstantOp::new(0.0));
    let to_celsius = graph.add(ListMapOp::new());
    graph.connect(readings, 0, to_celsius, 0).unwrap();
    graph.connect(dynamic_scale, 0, to_celsius, 1).unwrap();  // Dynamic scale!
    graph.connect(zero_offset, 0, to_celsius, 2).unwrap();

    // Compute statistics on converted temperatures
    let avg_temp = graph.add(ListAverageOp::new());
    let min_temp = graph.add(ListMinOp::new());
    let max_temp = graph.add(ListMaxOp::new());

    graph.connect(to_celsius, 0, avg_temp, 0).unwrap();
    graph.connect(to_celsius, 0, min_temp, 0).unwrap();
    graph.connect(to_celsius, 0, max_temp, 0).unwrap();

    // Compute spread (max - min) for alert detection
    let spread = graph.add(SubtractOp::new());
    graph.connect(max_temp, 0, spread, 0).unwrap();
    graph.connect(min_temp, 0, spread, 1).unwrap();

    // Access first and last readings with ListGet
    let get_first = graph.add(ListGetOp::new());
    let get_last = graph.add(ListGetOp::new());

    graph.connect(to_celsius, 0, get_first, 0).unwrap();
    graph.set_input_default(get_first, 1, Value::Int(0));   // First element

    graph.connect(to_celsius, 0, get_last, 0).unwrap();
    graph.set_input_default(get_last, 1, Value::Int(-1));   // Last element (negative index!)

    println!("  Pipeline structure:");
    println!("                                 SineWave");
    println!("                                    │");
    println!("    FloatList ─▶ ListMap ──────────▶+ scale ─┬─▶ Average ─▶ avg");
    println!("       [1.2, 2.5, 1.8, 3.1, 2.2]             ├─▶ Min ─┬───▶ min");
    println!("                                             ├─▶ Max ─┼───▶ max");
    println!("                                             │        └─▶ Subtract ─▶ spread");
    println!("                                             ├─▶ ListGet[0] ─▶ first");
    println!("                                             └─▶ ListGet[-1] ─▶ last");
    println!();

    // Show negative indexing
    println!("  Note: ListGet[-1] accesses the LAST element (Python-style negative indexing)");
    println!();

    // Simulate readings at different times
    println!("  {:>6}  {:>6}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
             "Time", "Scale", "Avg(°C)", "Min(°C)", "Max(°C)", "First", "Last");
    println!("  {:->6}  {:->6}  {:->8}  {:->8}  {:->8}  {:->8}  {:->8}", "", "", "", "", "", "", "");

    for frame in 0..8 {
        let mut ctx = EvalContext::new();
        ctx.time = frame as f64 * 0.25;

        let scale = graph.evaluate(dynamic_scale, 0, &ctx).unwrap().as_float().unwrap();
        let avg = graph.evaluate(avg_temp, 0, &ctx).unwrap().as_float().unwrap();
        let min = graph.evaluate(min_temp, 0, &ctx).unwrap().as_float().unwrap();
        let max = graph.evaluate(max_temp, 0, &ctx).unwrap().as_float().unwrap();
        let first = graph.evaluate(get_first, 0, &ctx).unwrap().as_float().unwrap();
        let last = graph.evaluate(get_last, 0, &ctx).unwrap().as_float().unwrap();

        println!("  {:>6.2}  {:>6.1}  {:>8.2}  {:>8.2}  {:>8.2}  {:>8.2}  {:>8.2}",
                 ctx.time, scale, avg, min, max, first, last);
    }
}
