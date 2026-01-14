//! Example 31: Collection Types
//!
//! This example demonstrates Flux's expanded collection system:
//! - Multiple list types: IntList, Vec3List, ColorList
//! - Polymorphic operators that work with ANY list type
//! - Type-specific operators for domain operations
//! - Conversion operators for transforming between types
//!
//! Key insight: The collection system uses a hybrid strategy where
//! generic operators (ListLength, ListGet) work polymorphically,
//! while type-specific operators (IntListRange, Vec3ListCentroid)
//! provide optimized domain-specific operations.
//!
//! Run with: cargo run --example 31_collection_types

use flux_core::{EvalContext, Value};
use flux_core::value::Color;
use flux_graph::Graph;
use flux_operators::{
    // Polymorphic list operators (work with any list type)
    ListLengthOp, ListReverseOp, ListFirstOp, ListLastOp,
    // Float list operators
    FloatListOp, ListSumOp, ListAverageOp,
    // Int list operators
    IntListOp, IntListSumOp, IntListMinOp, IntListMaxOp, IntListRangeOp,
    // Vec3 list operators
    Vec3ListOp, Vec3ListNormalizeOp, Vec3ListCentroidOp, Vec3ListBoundsOp,
    // Color list operators
    ColorListOp, ColorListSampleOp, ColorListBlendOp,
    // Conversion operators
    IntListToFloatListOp, FloatListToIntListOp,
    Vec3ListFlattenOp, FloatListToVec3ListOp,
    ColorListToVec4ListOp, Vec4ListToColorListOp,
};

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║ Demo 31: Collection Types                          ║");
    println!("╚════════════════════════════════════════════════════╝\n");

    demo_polymorphic_operators();
    demo_int_list_operations();
    demo_vec3_list_operations();
    demo_color_list_operations();
    demo_conversion_operators();

    println!("\n=== Summary ===\n");
    println!("Flux supports 8 list types with a hybrid operator strategy:\n");
    println!("  List Types:");
    println!("    FloatList, IntList, BoolList, StringList");
    println!("    Vec2List, Vec3List, Vec4List, ColorList\n");
    println!("  Polymorphic Operators (work with any list):");
    println!("    ListLength, ListGet, ListSlice, ListConcat");
    println!("    ListReverse, ListFirst, ListLast\n");
    println!("  Type-Specific Operators:");
    println!("    IntList:   IntListRange, IntListSum, IntListMin/Max");
    println!("    Vec3List:  Vec3ListNormalize, Vec3ListCentroid, Vec3ListBounds");
    println!("    ColorList: ColorListSample, ColorListBlend\n");
    println!("  Conversions:");
    println!("    IntList ↔ FloatList, Vec3List ↔ FloatList (flatten/group)");
    println!("    ColorList ↔ Vec4List\n");
}

/// Demonstrates that polymorphic operators work with any list type
fn demo_polymorphic_operators() {
    println!("=== Part 1: Polymorphic Operators ===\n");
    println!("  The same operators work with ANY list type:");
    println!();

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // Create lists of different types
    let float_list = graph.add(FloatListOp::new());
    graph.set_input_default(float_list, 0, Value::float_list(vec![1.5, 2.5, 3.5, 4.5]));

    let int_list = graph.add(IntListOp::new());
    graph.set_input_default(int_list, 0, Value::int_list(vec![10, 20, 30, 40, 50]));

    let vec3_list = graph.add(Vec3ListOp::new());
    graph.set_input_default(vec3_list, 0, Value::vec3_list(vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]));

    // ListLength works with all types
    let len_float = graph.add(ListLengthOp::new());
    let len_int = graph.add(ListLengthOp::new());
    let len_vec3 = graph.add(ListLengthOp::new());

    graph.connect(float_list, 0, len_float, 0).unwrap();
    graph.connect(int_list, 0, len_int, 0).unwrap();
    graph.connect(vec3_list, 0, len_vec3, 0).unwrap();

    let l1 = graph.evaluate(len_float, 0, &ctx).unwrap().as_int().unwrap();
    let l2 = graph.evaluate(len_int, 0, &ctx).unwrap().as_int().unwrap();
    let l3 = graph.evaluate(len_vec3, 0, &ctx).unwrap().as_int().unwrap();

    println!("  ListLength on FloatList[1.5, 2.5, 3.5, 4.5]: {}", l1);
    println!("  ListLength on IntList[10, 20, 30, 40, 50]:   {}", l2);
    // Note: Vec3List auto-coerces to FloatList (flattening xyz values)
    // 3 vectors × 3 components = 9 floats after flattening
    println!("  ListLength on Vec3List (3×3 flattened):     {}", l3);
    println!();

    // ListFirst/ListLast work polymorphically
    let first_float = graph.add(ListFirstOp::new());
    let last_int = graph.add(ListLastOp::new());

    graph.connect(float_list, 0, first_float, 0).unwrap();
    graph.connect(int_list, 0, last_int, 0).unwrap();

    let f = graph.evaluate(first_float, 0, &ctx).unwrap().as_float().unwrap();
    let l = graph.evaluate(last_int, 0, &ctx).unwrap().as_int().unwrap();

    println!("  ListFirst on FloatList: {}", f);
    println!("  ListLast on IntList:    {}", l);
    println!();

    // ListReverse works with any list
    let reverse_int = graph.add(ListReverseOp::new());
    graph.connect(int_list, 0, reverse_int, 0).unwrap();

    let reversed = graph.evaluate(reverse_int, 0, &ctx).unwrap();
    if let Value::IntList(vals) = reversed {
        println!("  ListReverse on IntList: {:?}", vals);
    }
    println!();
}

/// Demonstrates IntList-specific operators
fn demo_int_list_operations() {
    println!("=== Part 2: IntList Operations ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // IntListRange: Generate a sequence
    let range_op = graph.add(IntListRangeOp::new());
    graph.set_input_default(range_op, 0, Value::Int(0));   // Start
    graph.set_input_default(range_op, 1, Value::Int(10));  // End (exclusive)
    graph.set_input_default(range_op, 2, Value::Int(2));   // Step

    let range = graph.evaluate(range_op, 0, &ctx).unwrap();
    if let Value::IntList(vals) = range.clone() {
        println!("  IntListRange(0, 10, step=2): {:?}", vals);
    }

    // Create an IntList for aggregation
    let int_list = graph.add(IntListOp::new());
    graph.set_input_default(int_list, 0, Value::int_list(vec![15, 8, 23, 4, 19, 12]));

    // IntList aggregations
    let sum_op = graph.add(IntListSumOp::new());
    let min_op = graph.add(IntListMinOp::new());
    let max_op = graph.add(IntListMaxOp::new());

    graph.connect(int_list, 0, sum_op, 0).unwrap();
    graph.connect(int_list, 0, min_op, 0).unwrap();
    graph.connect(int_list, 0, max_op, 0).unwrap();

    let sum = graph.evaluate(sum_op, 0, &ctx).unwrap().as_int().unwrap();
    let min = graph.evaluate(min_op, 0, &ctx).unwrap().as_int().unwrap();
    let max = graph.evaluate(max_op, 0, &ctx).unwrap().as_int().unwrap();

    println!();
    println!("  IntList: [15, 8, 23, 4, 19, 12]");
    println!("    Sum: {}", sum);
    println!("    Min: {}", min);
    println!("    Max: {}", max);

    // Demonstrate descending range
    let desc_range = graph.add(IntListRangeOp::new());
    graph.set_input_default(desc_range, 0, Value::Int(5));   // Start > End
    graph.set_input_default(desc_range, 1, Value::Int(0));   // End
    graph.set_input_default(desc_range, 2, Value::Int(1));   // Step

    let desc = graph.evaluate(desc_range, 0, &ctx).unwrap();
    if let Value::IntList(vals) = desc {
        println!();
        println!("  IntListRange(5, 0, step=1): {:?}  (descending)", vals);
    }
    println!();
}

/// Demonstrates Vec3List-specific operators for 3D geometry
fn demo_vec3_list_operations() {
    println!("=== Part 3: Vec3List Operations (3D Geometry) ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // Create a list of 3D points (e.g., vertices of a shape)
    let points = graph.add(Vec3ListOp::new());
    graph.set_input_default(points, 0, Value::vec3_list(vec![
        [0.0, 0.0, 0.0],    // Origin
        [2.0, 0.0, 0.0],    // Right
        [1.0, 2.0, 0.0],    // Top
        [1.0, 1.0, 1.0],    // Offset point
    ]));

    println!("  Input points:");
    println!("    [0, 0, 0], [2, 0, 0], [1, 2, 0], [1, 1, 1]");
    println!();

    // Vec3ListCentroid: Find center of mass
    let centroid_op = graph.add(Vec3ListCentroidOp::new());
    graph.connect(points, 0, centroid_op, 0).unwrap();

    let centroid = graph.evaluate(centroid_op, 0, &ctx).unwrap();
    if let Value::Vec3(c) = centroid {
        println!("  Centroid: [{:.2}, {:.2}, {:.2}]", c[0], c[1], c[2]);
    }

    // Vec3ListBounds: Get bounding box
    let bounds_op = graph.add(Vec3ListBoundsOp::new());
    graph.connect(points, 0, bounds_op, 0).unwrap();

    let min_bounds = graph.evaluate(bounds_op, 0, &ctx).unwrap();  // Output 0: min
    let max_bounds = graph.evaluate(bounds_op, 1, &ctx).unwrap();  // Output 1: max

    if let (Value::Vec3(min), Value::Vec3(max)) = (min_bounds, max_bounds) {
        println!("  Bounds:");
        println!("    Min: [{:.1}, {:.1}, {:.1}]", min[0], min[1], min[2]);
        println!("    Max: [{:.1}, {:.1}, {:.1}]", max[0], max[1], max[2]);
    }

    // Vec3ListNormalize: Normalize all vectors
    let vectors = graph.add(Vec3ListOp::new());
    graph.set_input_default(vectors, 0, Value::vec3_list(vec![
        [3.0, 0.0, 0.0],   // Length 3
        [0.0, 4.0, 0.0],   // Length 4
        [0.0, 0.0, 5.0],   // Length 5
    ]));

    let normalize_op = graph.add(Vec3ListNormalizeOp::new());
    graph.connect(vectors, 0, normalize_op, 0).unwrap();

    let normalized = graph.evaluate(normalize_op, 0, &ctx).unwrap();
    println!();
    println!("  Normalize vectors [3,0,0], [0,4,0], [0,0,5]:");
    if let Value::Vec3List(vecs) = normalized {
        for v in vecs.iter() {
            let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
            println!("    [{:.1}, {:.1}, {:.1}] (length: {:.1})", v[0], v[1], v[2], len);
        }
    }
    println!();
}

/// Demonstrates ColorList-specific operators for color palettes
fn demo_color_list_operations() {
    println!("=== Part 4: ColorList Operations (Color Palettes) ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // Create a color palette (gradient from red to blue)
    let palette = graph.add(ColorListOp::new());
    graph.set_input_default(palette, 0, Value::color_list(vec![
        Color::rgba(1.0, 0.0, 0.0, 1.0),  // Red
        Color::rgba(1.0, 1.0, 0.0, 1.0),  // Yellow
        Color::rgba(0.0, 1.0, 0.0, 1.0),  // Green
        Color::rgba(0.0, 1.0, 1.0, 1.0),  // Cyan
        Color::rgba(0.0, 0.0, 1.0, 1.0),  // Blue
    ]));

    println!("  Color palette: Red → Yellow → Green → Cyan → Blue");
    println!();

    // ColorListSample: Interpolate colors at positions
    println!("  Sampling at different positions (0.0 to 1.0):");

    for pos in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let sample_op = graph.add(ColorListSampleOp::new());
        graph.connect(palette, 0, sample_op, 0).unwrap();
        graph.set_input_default(sample_op, 1, Value::Float(pos));

        let color = graph.evaluate(sample_op, 0, &ctx).unwrap();
        if let Value::Color(c) = color {
            let name = match pos {
                x if x < 0.1 => "Red",
                x if (x - 0.25).abs() < 0.01 => "Orange",
                x if (x - 0.5).abs() < 0.01 => "Green",
                x if (x - 0.75).abs() < 0.01 => "Teal",
                _ => "Blue",
            };
            println!("    pos={:.2}: RGB({:.2}, {:.2}, {:.2}) ~ {}",
                     pos, c.r, c.g, c.b, name);
        }
    }

    // ColorListBlend: Average all colors
    let blend_op = graph.add(ColorListBlendOp::new());
    graph.connect(palette, 0, blend_op, 0).unwrap();

    let blended = graph.evaluate(blend_op, 0, &ctx).unwrap();
    if let Value::Color(c) = blended {
        println!();
        println!("  ColorListBlend (average): RGB({:.2}, {:.2}, {:.2})", c.r, c.g, c.b);
    }
    println!();
}

/// Demonstrates conversion operators between list types
fn demo_conversion_operators() {
    println!("=== Part 5: Conversion Operators ===\n");

    let mut graph = Graph::new();
    let ctx = EvalContext::new();

    // --- IntList ↔ FloatList ---
    println!("  IntList ↔ FloatList:");

    let int_list = graph.add(IntListOp::new());
    graph.set_input_default(int_list, 0, Value::int_list(vec![1, 2, 3, 4, 5]));

    let to_float = graph.add(IntListToFloatListOp::new());
    graph.connect(int_list, 0, to_float, 0).unwrap();

    let float_result = graph.evaluate(to_float, 0, &ctx).unwrap();
    if let Value::FloatList(vals) = float_result {
        println!("    IntList[1,2,3,4,5] → FloatList{:?}", vals);
    }

    // FloatList → IntList (truncates)
    let float_list = graph.add(FloatListOp::new());
    graph.set_input_default(float_list, 0, Value::float_list(vec![1.9, 2.1, 3.7, 4.2]));

    let to_int = graph.add(FloatListToIntListOp::new());
    graph.connect(float_list, 0, to_int, 0).unwrap();

    let int_result = graph.evaluate(to_int, 0, &ctx).unwrap();
    if let Value::IntList(vals) = int_result {
        println!("    FloatList[1.9,2.1,3.7,4.2] → IntList{:?} (truncated)", vals);
    }
    println!();

    // --- Vec3List ↔ FloatList ---
    println!("  Vec3List ↔ FloatList:");

    let vec3_list = graph.add(Vec3ListOp::new());
    graph.set_input_default(vec3_list, 0, Value::vec3_list(vec![
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
    ]));

    // Flatten: Vec3List → FloatList
    let flatten = graph.add(Vec3ListFlattenOp::new());
    graph.connect(vec3_list, 0, flatten, 0).unwrap();

    let flat = graph.evaluate(flatten, 0, &ctx).unwrap();
    if let Value::FloatList(vals) = flat {
        println!("    Vec3List[[1,2,3],[4,5,6]] → FloatList{:?}", vals);
    }

    // Group: FloatList → Vec3List
    let float_data = graph.add(FloatListOp::new());
    graph.set_input_default(float_data, 0, Value::float_list(vec![
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0
    ]));

    let to_vec3 = graph.add(FloatListToVec3ListOp::new());
    graph.connect(float_data, 0, to_vec3, 0).unwrap();

    let vec3_result = graph.evaluate(to_vec3, 0, &ctx).unwrap();
    if let Value::Vec3List(vecs) = vec3_result {
        print!("    FloatList[10..60] → Vec3List[");
        for (i, v) in vecs.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("[{:.0},{:.0},{:.0}]", v[0], v[1], v[2]);
        }
        println!("]");
    }
    println!();

    // --- ColorList ↔ Vec4List ---
    println!("  ColorList ↔ Vec4List:");

    let colors = graph.add(ColorListOp::new());
    graph.set_input_default(colors, 0, Value::color_list(vec![
        Color::rgba(1.0, 0.0, 0.0, 1.0),  // Red
        Color::rgba(0.0, 1.0, 0.0, 0.5),  // Green (50% alpha)
    ]));

    let to_vec4 = graph.add(ColorListToVec4ListOp::new());
    graph.connect(colors, 0, to_vec4, 0).unwrap();

    let vec4_result = graph.evaluate(to_vec4, 0, &ctx).unwrap();
    if let Value::Vec4List(vecs) = vec4_result {
        print!("    ColorList[Red, Green] → Vec4List[");
        for (i, v) in vecs.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("[{:.1},{:.1},{:.1},{:.1}]", v[0], v[1], v[2], v[3]);
        }
        println!("]");
    }

    // Vec4List → ColorList
    // Set the default directly on the conversion operator's input
    let to_color = graph.add(Vec4ListToColorListOp::new());
    graph.set_input_default(to_color, 0, Value::vec4_list(vec![
        [0.0, 0.0, 1.0, 1.0],  // Blue
        [1.0, 1.0, 0.0, 0.8],  // Yellow (80% alpha)
    ]));

    let color_result = graph.evaluate(to_color, 0, &ctx).unwrap();
    if let Value::ColorList(cols) = color_result {
        print!("    Vec4List[[0,0,1,1],[1,1,0,.8]] → ColorList[");
        for (i, c) in cols.iter().enumerate() {
            if i > 0 { print!(", "); }
            let name = if c.b > 0.9 { "Blue" } else { "Yellow" };
            print!("{}({:.0}%)", name, c.a * 100.0);
        }
        println!("]");
    }
    println!();

    // --- Practical Pipeline: Generate points, transform, analyze ---
    println!("  Practical Pipeline:");
    println!("    IntListRange → IntListToFloatList → ListMap → Statistics");
    println!();

    // Generate indices 0-9
    let range = graph.add(IntListRangeOp::new());
    graph.set_input_default(range, 0, Value::Int(0));
    graph.set_input_default(range, 1, Value::Int(10));
    graph.set_input_default(range, 2, Value::Int(1));

    // Convert to float for math
    let convert = graph.add(IntListToFloatListOp::new());
    graph.connect(range, 0, convert, 0).unwrap();

    // Calculate statistics
    let sum_op = graph.add(ListSumOp::new());
    let avg_op = graph.add(ListAverageOp::new());
    graph.connect(convert, 0, sum_op, 0).unwrap();
    graph.connect(convert, 0, avg_op, 0).unwrap();

    let sum = graph.evaluate(sum_op, 0, &ctx).unwrap().as_float().unwrap();
    let avg = graph.evaluate(avg_op, 0, &ctx).unwrap().as_float().unwrap();

    println!("    Range [0..10] → Sum: {:.0}, Average: {:.1}", sum, avg);
    println!();
}
