//! Demo 18: Phase 3 Operators Showcase
//!
//! This example demonstrates all 94 operators from Phase 3:
//! - Math operators (Floor, Ceil, Round, Lerp, SmoothStep, Random, PerlinNoise)
//! - Logic operators (And, Or, Not, Xor)
//! - Integer operators (IntAdd, IntMultiply, IntToFloat)
//! - Vector operators (Vec2/Vec3 Compose, Decompose, Cross, Dot, Normalize)
//! - Color operators (RGBA, HSV->RGB, AdjustBrightness)
//! - Time operators (Time, DeltaTime, Frame, SawWave, PulseWave)
//! - Flow operators (Switch, Gate, Counter, GetFloatVar)
//! - String operators (Concat, Format, Length, FloatToString)
//! - List operators (FloatList, Length, Sum, Average, Map)
//! - Utility operators (Print, Passthrough, TypeOf, IsConnected)
//!
//! Run with: `cargo run --example 18_phase3_operators`

use std::f32::consts::PI;

use flux_operators::{
    // Color operators
    color::{AdjustBrightnessOp, HsvToRgbOp, RgbaColorOp},
    // Flow operators
    flow::{CounterOp, GateOp, GetFloatVarOp, SwitchOp},
    // List operators
    list::{FloatListOp, ListAverageOp, ListLengthOp, ListMapOp, ListSumOp},
    // Logic operators
    logic::{AndOp, IntAddOp, IntMultiplyOp, IntToFloatOp, NotOp, OrOp, XorOp},
    // Math operators
    math::{CeilOp, FloorOp, LerpOp, PerlinNoiseOp, RandomOp, RoundOp, SmoothStepOp},
    // String operators
    string::{FloatToStringOp, StringConcatOp, StringFormatOp, StringLengthOp},
    // Time operators
    time::{DeltaTimeOp, FrameOp, PulseWaveOp, SawWaveOp, TimeOp},
    // Utility operators
    util::{IsConnectedOp, PassthroughOp, PrintOp, TypeOfOp},
    // Vector operators
    vector::{Vec2ComposeOp, Vec2DecomposeOp, Vec3CrossOp, Vec3DotOp, Vec3NormalizeOp},
};
use flux_core::{Color, EvalContext, Id, Operator, Value};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 18: Phase 3 Operators (94 total)  ║");
    println!("╚════════════════════════════════════════╝\n");

    println!("--- Math Operators ---");

    let ctx = EvalContext::new();
    let no_conn = |_: Id, _: usize| Value::Float(0.0);

    // Floor, Ceil, Round
    let mut floor_op = FloorOp::new();
    let mut ceil_op = CeilOp::new();
    let mut round_op = RoundOp::new();

    floor_op.inputs_mut()[0].default = Value::Float(3.7);
    floor_op.compute(&ctx, &no_conn);
    println!(
        "  Floor(3.7) = {}",
        floor_op.outputs()[0].value.as_float().unwrap()
    );

    ceil_op.inputs_mut()[0].default = Value::Float(3.2);
    ceil_op.compute(&ctx, &no_conn);
    println!(
        "  Ceil(3.2) = {}",
        ceil_op.outputs()[0].value.as_float().unwrap()
    );

    round_op.inputs_mut()[0].default = Value::Float(3.5);
    round_op.compute(&ctx, &no_conn);
    println!(
        "  Round(3.5) = {}",
        round_op.outputs()[0].value.as_float().unwrap()
    );

    // Lerp
    let mut lerp_op = LerpOp::new();
    lerp_op.inputs_mut()[0].default = Value::Float(0.0); // A
    lerp_op.inputs_mut()[1].default = Value::Float(100.0); // B
    lerp_op.inputs_mut()[2].default = Value::Float(0.25); // T
    lerp_op.compute(&ctx, &no_conn);
    println!(
        "  Lerp(0, 100, 0.25) = {}",
        lerp_op.outputs()[0].value.as_float().unwrap()
    );

    // SmoothStep
    let mut smoothstep_op = SmoothStepOp::new();
    smoothstep_op.inputs_mut()[0].default = Value::Float(0.0); // Edge0
    smoothstep_op.inputs_mut()[1].default = Value::Float(1.0); // Edge1
    smoothstep_op.inputs_mut()[2].default = Value::Float(0.5); // X
    smoothstep_op.compute(&ctx, &no_conn);
    println!(
        "  SmoothStep(0, 1, 0.5) = {:.3}",
        smoothstep_op.outputs()[0].value.as_float().unwrap()
    );

    // Random (deterministic with seed)
    let mut random_op = RandomOp::new();
    random_op.inputs_mut()[0].default = Value::Float(42.0); // Seed
    random_op.compute(&ctx, &no_conn);
    println!(
        "  Random(seed=42) = {:.3}",
        random_op.outputs()[0].value.as_float().unwrap()
    );

    // Perlin Noise
    let mut perlin_op = PerlinNoiseOp::new();
    perlin_op.inputs_mut()[0].default = Value::Float(1.5); // X
    perlin_op.inputs_mut()[1].default = Value::Float(2.5); // Y
    perlin_op.compute(&ctx, &no_conn);
    println!(
        "  PerlinNoise(1.5, 2.5) = {:.3}",
        perlin_op.outputs()[0].value.as_float().unwrap()
    );

    println!("\n--- Logic Operators ---");

    let mut and_op = AndOp::new();
    and_op.inputs_mut()[0].default = Value::Bool(true);
    and_op.inputs_mut()[1].default = Value::Bool(false);
    and_op.compute(&ctx, &no_conn);
    println!(
        "  true AND false = {}",
        and_op.outputs()[0].value.as_bool().unwrap()
    );

    let mut or_op = OrOp::new();
    or_op.inputs_mut()[0].default = Value::Bool(true);
    or_op.inputs_mut()[1].default = Value::Bool(false);
    or_op.compute(&ctx, &no_conn);
    println!(
        "  true OR false = {}",
        or_op.outputs()[0].value.as_bool().unwrap()
    );

    let mut not_op = NotOp::new();
    not_op.inputs_mut()[0].default = Value::Bool(true);
    not_op.compute(&ctx, &no_conn);
    println!(
        "  NOT true = {}",
        not_op.outputs()[0].value.as_bool().unwrap()
    );

    let mut xor_op = XorOp::new();
    xor_op.inputs_mut()[0].default = Value::Bool(true);
    xor_op.inputs_mut()[1].default = Value::Bool(true);
    xor_op.compute(&ctx, &no_conn);
    println!(
        "  true XOR true = {}",
        xor_op.outputs()[0].value.as_bool().unwrap()
    );

    println!("\n--- Integer Operators ---");

    let mut int_add = IntAddOp::new();
    int_add.inputs_mut()[0].default = Value::Int(42);
    int_add.inputs_mut()[1].default = Value::Int(8);
    int_add.compute(&ctx, &no_conn);
    println!(
        "  42 + 8 = {}",
        int_add.outputs()[0].value.as_int().unwrap()
    );

    let mut int_mul = IntMultiplyOp::new();
    int_mul.inputs_mut()[0].default = Value::Int(7);
    int_mul.inputs_mut()[1].default = Value::Int(6);
    int_mul.compute(&ctx, &no_conn);
    println!(
        "  7 * 6 = {}",
        int_mul.outputs()[0].value.as_int().unwrap()
    );

    let mut int_to_float = IntToFloatOp::new();
    int_to_float.inputs_mut()[0].default = Value::Int(255);
    int_to_float.compute(&ctx, &no_conn);
    println!(
        "  IntToFloat(255) = {}",
        int_to_float.outputs()[0].value.as_float().unwrap()
    );

    println!("\n--- Vector Operators ---");

    let mut vec2_compose = Vec2ComposeOp::new();
    vec2_compose.inputs_mut()[0].default = Value::Float(3.0);
    vec2_compose.inputs_mut()[1].default = Value::Float(4.0);
    vec2_compose.compute(&ctx, &no_conn);
    let v2 = vec2_compose.outputs()[0].value.as_vec2().unwrap();
    println!("  Vec2Compose(3, 4) = [{}, {}]", v2[0], v2[1]);

    let mut vec2_decompose = Vec2DecomposeOp::new();
    vec2_decompose.inputs_mut()[0].default = Value::Vec2([5.0, 12.0]);
    vec2_decompose.compute(&ctx, &no_conn);
    println!(
        "  Vec2Decompose([5, 12]) = x:{}, y:{}",
        vec2_decompose.outputs()[0].value.as_float().unwrap(),
        vec2_decompose.outputs()[1].value.as_float().unwrap()
    );

    let mut cross = Vec3CrossOp::new();
    cross.inputs_mut()[0].default = Value::Vec3([1.0, 0.0, 0.0]);
    cross.inputs_mut()[1].default = Value::Vec3([0.0, 1.0, 0.0]);
    cross.compute(&ctx, &no_conn);
    let c = cross.outputs()[0].value.as_vec3().unwrap();
    println!("  Cross([1,0,0], [0,1,0]) = [{}, {}, {}]", c[0], c[1], c[2]);

    let mut dot = Vec3DotOp::new();
    dot.inputs_mut()[0].default = Value::Vec3([1.0, 2.0, 3.0]);
    dot.inputs_mut()[1].default = Value::Vec3([4.0, 5.0, 6.0]);
    dot.compute(&ctx, &no_conn);
    println!(
        "  Dot([1,2,3], [4,5,6]) = {}",
        dot.outputs()[0].value.as_float().unwrap()
    );

    let mut normalize = Vec3NormalizeOp::new();
    normalize.inputs_mut()[0].default = Value::Vec3([3.0, 4.0, 0.0]);
    normalize.compute(&ctx, &no_conn);
    let n = normalize.outputs()[0].value.as_vec3().unwrap();
    println!("  Normalize([3,4,0]) = [{:.2}, {:.2}, {:.2}]", n[0], n[1], n[2]);

    println!("\n--- Color Operators ---");

    let mut rgba = RgbaColorOp::new();
    rgba.inputs_mut()[0].default = Value::Float(1.0); // R
    rgba.inputs_mut()[1].default = Value::Float(0.5); // G
    rgba.inputs_mut()[2].default = Value::Float(0.25); // B
    rgba.inputs_mut()[3].default = Value::Float(1.0); // A
    rgba.compute(&ctx, &no_conn);
    let color = rgba.outputs()[0].value.as_color().unwrap();
    println!(
        "  RGBA(1, 0.5, 0.25, 1) = Color(r:{}, g:{}, b:{}, a:{})",
        color.r, color.g, color.b, color.a
    );

    let mut hsv_to_rgb = HsvToRgbOp::new();
    hsv_to_rgb.inputs_mut()[0].default = Value::Float(120.0); // H (green)
    hsv_to_rgb.inputs_mut()[1].default = Value::Float(1.0); // S
    hsv_to_rgb.inputs_mut()[2].default = Value::Float(1.0); // V
    hsv_to_rgb.compute(&ctx, &no_conn);
    let rgb_color = hsv_to_rgb.outputs()[0].value.as_color().unwrap();
    println!(
        "  HSV(120°, 1, 1) = RGB({:.1}, {:.1}, {:.1})",
        rgb_color.r, rgb_color.g, rgb_color.b
    );

    let mut brightness = AdjustBrightnessOp::new();
    brightness.inputs_mut()[0].default = Value::Color(Color::rgb(0.5, 0.5, 0.5));
    brightness.inputs_mut()[1].default = Value::Float(0.3);
    brightness.compute(&ctx, &no_conn);
    let bright_color = brightness.outputs()[0].value.as_color().unwrap();
    println!(
        "  AdjustBrightness(gray, +0.3) = ({:.2}, {:.2}, {:.2})",
        bright_color.r, bright_color.g, bright_color.b
    );

    println!("\n--- Time Operators ---");

    let mut time_ctx = EvalContext::new();
    time_ctx.time = 2.5;
    time_ctx.delta_time = 0.016;
    time_ctx.frame = 150;

    let mut time_op = TimeOp::new();
    time_op.compute(&time_ctx, &no_conn);
    println!(
        "  Time = {}s",
        time_op.outputs()[0].value.as_float().unwrap()
    );

    let mut delta_op = DeltaTimeOp::new();
    delta_op.compute(&time_ctx, &no_conn);
    println!(
        "  DeltaTime = {}s",
        delta_op.outputs()[0].value.as_float().unwrap()
    );

    let mut frame_op = FrameOp::new();
    frame_op.compute(&time_ctx, &no_conn);
    println!("  Frame = {}", frame_op.outputs()[0].value.as_int().unwrap());

    let mut saw_op = SawWaveOp::new();
    saw_op.inputs_mut()[0].default = Value::Float(1.0); // Frequency
    saw_op.inputs_mut()[1].default = Value::Float(1.0); // Amplitude
    saw_op.compute(&time_ctx, &no_conn);
    println!(
        "  SawWave(1Hz) at t=2.5s = {:.3}",
        saw_op.outputs()[0].value.as_float().unwrap()
    );

    let mut pulse_op = PulseWaveOp::new();
    pulse_op.inputs_mut()[0].default = Value::Float(2.0); // Frequency
    pulse_op.inputs_mut()[1].default = Value::Float(0.5); // Duty cycle
    pulse_op.compute(&time_ctx, &no_conn);
    println!(
        "  PulseWave(2Hz, 50% duty) at t=2.5s = {}",
        pulse_op.outputs()[0].value.as_float().unwrap()
    );

    println!("\n--- Flow Operators ---");

    let mut switch_op = SwitchOp::new();
    switch_op.inputs_mut()[0].default = Value::Bool(false); // Condition
    switch_op.inputs_mut()[1].default = Value::Float(100.0); // True value
    switch_op.inputs_mut()[2].default = Value::Float(200.0); // False value
    switch_op.compute(&ctx, &no_conn);
    println!(
        "  Switch(false, 100, 200) = {}",
        switch_op.outputs()[0].value.as_float().unwrap()
    );

    let mut gate_op = GateOp::new();
    gate_op.inputs_mut()[0].default = Value::Float(42.0); // Value
    gate_op.inputs_mut()[1].default = Value::Bool(true); // Open
    gate_op.compute(&ctx, &no_conn);
    println!(
        "  Gate(42, open=true) = {}",
        gate_op.outputs()[0].value.as_float().unwrap()
    );

    let mut counter_op = CounterOp::new();
    counter_op.inputs_mut()[0].default = Value::Bool(true); // Trigger
    counter_op.inputs_mut()[1].default = Value::Bool(false); // Reset
    for i in 0..5 {
        counter_op.compute(&ctx, &no_conn);
        if i == 4 {
            println!(
                "  Counter (5 triggers) = {}",
                counter_op.outputs()[0].value.as_int().unwrap()
            );
        }
    }

    // Context variables
    let mut ctx_vars = EvalContext::new();
    ctx_vars.set_float_var("myVar", 99.5);

    let mut get_var = GetFloatVarOp::new();
    get_var.inputs_mut()[0].default = Value::String("myVar".to_string());
    get_var.inputs_mut()[1].default = Value::Float(0.0);
    get_var.compute(&ctx_vars, &no_conn);
    println!(
        "  GetFloatVar('myVar') = {}",
        get_var.outputs()[0].value.as_float().unwrap()
    );

    println!("\n--- String Operators ---");

    let mut concat = StringConcatOp::new();
    concat.inputs_mut()[0].default = Value::String("Hello, ".to_string());
    concat.inputs_mut()[1].default = Value::String("World!".to_string());
    concat.compute(&ctx, &no_conn);
    println!(
        "  Concat('Hello, ', 'World!') = '{}'",
        concat.outputs()[0].value.as_string().unwrap()
    );

    let mut format_op = StringFormatOp::new();
    format_op.inputs_mut()[0].default = Value::String("Value: {}".to_string());
    format_op.inputs_mut()[1].default = Value::Float(42.5);
    format_op.compute(&ctx, &no_conn);
    println!(
        "  Format('Value: {{}}', 42.5) = '{}'",
        format_op.outputs()[0].value.as_string().unwrap()
    );

    let mut length = StringLengthOp::new();
    length.inputs_mut()[0].default = Value::String("Testing".to_string());
    length.compute(&ctx, &no_conn);
    println!(
        "  Length('Testing') = {}",
        length.outputs()[0].value.as_int().unwrap()
    );

    let mut float_to_str = FloatToStringOp::new();
    float_to_str.inputs_mut()[0].default = Value::Float(PI);
    float_to_str.inputs_mut()[1].default = Value::Int(2); // Precision
    float_to_str.compute(&ctx, &no_conn);
    println!(
        "  FloatToString(3.14159, 2) = '{}'",
        float_to_str.outputs()[0].value.as_string().unwrap()
    );

    println!("\n--- List Operators ---");

    // FloatListOp uses multi-input, so we set the default to a pre-built list
    let mut list_op = FloatListOp::new();
    list_op.inputs_mut()[0].default = Value::float_list(vec![1.0, 2.0, 3.0, 4.0]);
    list_op.compute(&ctx, &no_conn);
    let list = list_op.outputs()[0].value.as_float_list().unwrap();
    println!("  FloatList([1,2,3,4]) = {:?}", list);

    let mut list_len = ListLengthOp::new();
    list_len.inputs_mut()[0].default = Value::float_list(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    list_len.compute(&ctx, &no_conn);
    println!(
        "  ListLength([1,2,3,4,5]) = {}",
        list_len.outputs()[0].value.as_int().unwrap()
    );

    let mut list_sum = ListSumOp::new();
    list_sum.inputs_mut()[0].default = Value::float_list(vec![10.0, 20.0, 30.0]);
    list_sum.compute(&ctx, &no_conn);
    println!(
        "  ListSum([10,20,30]) = {}",
        list_sum.outputs()[0].value.as_float().unwrap()
    );

    let mut list_avg = ListAverageOp::new();
    list_avg.inputs_mut()[0].default = Value::float_list(vec![2.0, 4.0, 6.0, 8.0]);
    list_avg.compute(&ctx, &no_conn);
    println!(
        "  ListAverage([2,4,6,8]) = {}",
        list_avg.outputs()[0].value.as_float().unwrap()
    );

    let mut list_map = ListMapOp::new();
    list_map.inputs_mut()[0].default = Value::float_list(vec![1.0, 2.0, 3.0]);
    list_map.inputs_mut()[1].default = Value::Float(2.0); // Multiplier
    list_map.inputs_mut()[2].default = Value::Float(10.0); // Offset
    list_map.compute(&ctx, &no_conn);
    let mapped = list_map.outputs()[0].value.as_float_list().unwrap();
    println!("  ListMap([1,2,3], *2, +10) = {:?}", mapped);

    println!("\n--- Utility Operators ---");

    let mut print_op = PrintOp::new();
    print_op.inputs_mut()[0].default = Value::Float(123.456);
    print_op.inputs_mut()[1].default = Value::String("Debug".to_string());
    print_op.inputs_mut()[2].default = Value::Bool(true);
    print_op.compute(&ctx, &no_conn);
    println!("  Print output: '{}'", print_op.last_message());

    let mut passthrough = PassthroughOp::new();
    passthrough.inputs_mut()[0].default = Value::Float(999.0);
    passthrough.compute(&ctx, &no_conn);
    println!(
        "  Passthrough(999) = {}",
        passthrough.outputs()[0].value.as_float().unwrap()
    );

    let mut typeof_op = TypeOfOp::new();
    typeof_op.inputs_mut()[0].default = Value::Vec3([1.0, 2.0, 3.0]);
    typeof_op.compute(&ctx, &no_conn);
    println!(
        "  TypeOf(Vec3) = '{}'",
        typeof_op.outputs()[0].value.as_string().unwrap()
    );

    typeof_op.inputs_mut()[0].default = Value::String("hello".to_string());
    typeof_op.compute(&ctx, &no_conn);
    println!(
        "  TypeOf(String) = '{}'",
        typeof_op.outputs()[0].value.as_string().unwrap()
    );

    let mut is_conn = IsConnectedOp::new();
    is_conn.compute(&ctx, &no_conn);
    println!(
        "  IsConnected (no connection) = {}",
        is_conn.outputs()[0].value.as_bool().unwrap()
    );

    is_conn.inputs_mut()[0].connection = Some((Id::new(), 0));
    is_conn.compute(&ctx, &no_conn);
    println!(
        "  IsConnected (with connection) = {}",
        is_conn.outputs()[0].value.as_bool().unwrap()
    );

    println!("\n--- Operator Count Summary ---");
    println!("  Math:     35 operators (Floor, Ceil, Round, Lerp, SmoothStep, Random, PerlinNoise, etc.)");
    println!("  Logic:     6 operators (And, Or, Not, Xor, All, Any)");
    println!("  Integer:   6 operators (IntAdd, IntSubtract, IntMultiply, IntDivide, IntModulo, IntToFloat)");
    println!("  Vector:   15 operators (Vec2/Vec3/Vec4 Compose, Decompose, Add, Cross, Dot, Normalize, etc.)");
    println!("  Color:     8 operators (RGBA, HSV<->RGB, Blend, Gradient, Brightness, Saturation)");
    println!("  Time:     10 operators (Time, DeltaTime, Frame, SineWave, SawWave, PulseWave, Spring)");
    println!("  Flow:     12 operators (Switch, Gate, Delay, Previous, Changed, Trigger, Counter, Variables)");
    println!("  String:    8 operators (Concat, Format, Length, SubString, Split, Contains, Conversions)");
    println!("  List:      8 operators (FloatList, Length, Get, Sum, Average, Min, Max, Map)");
    println!("  Utility:   6 operators (Print, Passthrough, Comment, Bookmark, TypeOf, IsConnected)");
    println!("  ─────────────────────────────────────────");
    println!("  TOTAL:   94 operators");
}
