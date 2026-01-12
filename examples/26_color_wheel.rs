//! Example 26: HSV Color Wheel Animation
//!
//! This example demonstrates the complete color pipeline in Flux:
//! - HSV to RGB color space conversion
//! - Animated hue rotation using SawWave oscillator
//! - Triadic color harmony (colors 120 degrees apart)
//! - Color blending and adjustment operations
//!
//! Run with: cargo run --example 26_color_wheel

use flux_core::{EvalContext, Operator, Value};
use flux_operators::{
    BlendColorsOp, HsvToRgbOp, RgbToHsvOp, SawWaveOp,
    AdjustBrightnessOp, AdjustSaturationOp,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 26: HSV Color Wheel Animation     ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_hsv_basics();
    demo_color_wheel();
    demo_triadic_harmony();
    demo_color_blending();

    println!("\n=== Summary ===\n");
    println!("Color pipeline techniques demonstrated:");
    println!();
    println!("  HSV Color Space:");
    println!("    - H (Hue): 0-360 degrees around the color wheel");
    println!("    - S (Saturation): 0-1, gray to vivid");
    println!("    - V (Value): 0-1, black to bright");
    println!();
    println!("  Color Harmony:");
    println!("    - Triadic: 3 colors, 120 degrees apart");
    println!("    - Complementary: 2 colors, 180 degrees apart");
    println!("    - Analogous: Adjacent colors, ~30 degrees apart");
    println!();
    println!("  Flux Color Operators:");
    println!("    - HsvToRgb: Create colors from H, S, V inputs");
    println!("    - RgbToHsv: Decompose colors to H, S, V outputs");
    println!("    - BlendColors: Linear interpolation between colors");
    println!("    - AdjustBrightness/Saturation: Modify color properties");
}

/// Demo basic HSV to RGB conversion
fn demo_hsv_basics() {
    println!("=== Part 1: HSV Color Basics ===\n");

    let mut hsv_to_rgb = HsvToRgbOp::new();
    let ctx = EvalContext::new();

    println!("HSV to RGB conversion at full saturation and value:\n");
    println!("  {:>6}  {:>12}  {:>18}", "Hue", "Color Name", "RGB Values");
    println!("  {:->6}  {:->12}  {:->18}", "", "", "");

    let color_names = [
        (0.0, "Red"),
        (60.0, "Yellow"),
        (120.0, "Green"),
        (180.0, "Cyan"),
        (240.0, "Blue"),
        (300.0, "Magenta"),
    ];

    for (hue, name) in color_names {
        hsv_to_rgb.inputs_mut()[0].default = Value::Float(hue);
        hsv_to_rgb.inputs_mut()[1].default = Value::Float(1.0); // S
        hsv_to_rgb.inputs_mut()[2].default = Value::Float(1.0); // V
        hsv_to_rgb.inputs_mut()[3].default = Value::Float(1.0); // A

        hsv_to_rgb.compute(&ctx, &|_, _| Value::Float(0.0));
        let color = hsv_to_rgb.outputs()[0].value.as_color().unwrap();

        println!("  {:>6.0}  {:>12}  ({:.2}, {:.2}, {:.2})",
                 hue, name, color.r, color.g, color.b);
    }
}

/// Demo animated color wheel using SawWave
fn demo_color_wheel() {
    println!("\n=== Part 2: Animated Color Wheel ===\n");

    let mut saw_wave = SawWaveOp::new();
    let mut hsv_to_rgb = HsvToRgbOp::new();

    // Configure SawWave for hue rotation: 0-360 degrees over 2 seconds
    saw_wave.inputs_mut()[0].default = Value::Float(0.5);   // 0.5 Hz = 2 second cycle
    saw_wave.inputs_mut()[1].default = Value::Float(180.0); // Amplitude (half range)
    saw_wave.inputs_mut()[2].default = Value::Float(0.0);   // Phase
    saw_wave.inputs_mut()[3].default = Value::Float(180.0); // Offset (center at 180)

    hsv_to_rgb.inputs_mut()[1].default = Value::Float(1.0); // Full saturation
    hsv_to_rgb.inputs_mut()[2].default = Value::Float(1.0); // Full value
    hsv_to_rgb.inputs_mut()[3].default = Value::Float(1.0); // Full alpha

    println!("Simulating color wheel rotation over 2 seconds:\n");
    println!("  {:>6}  {:>8}  {:>18}  Visual", "Time", "Hue", "RGB");
    println!("  {:->6}  {:->8}  {:->18}  {:->6}", "", "", "", "");

    for frame in 0..=8 {
        let t = frame as f64 * 0.25;
        let mut ctx = EvalContext::new();
        ctx.time = t;

        saw_wave.compute(&ctx, &|_, _| Value::Float(0.0));
        let hue = saw_wave.outputs()[0].value.as_float().unwrap();

        hsv_to_rgb.inputs_mut()[0].default = Value::Float(hue);
        hsv_to_rgb.compute(&ctx, &|_, _| Value::Float(0.0));
        let color = hsv_to_rgb.outputs()[0].value.as_color().unwrap();

        // ASCII color visualization
        let visual = color_to_ascii(color.r, color.g, color.b);

        println!("  {:>6.2}  {:>8.1}  ({:.2}, {:.2}, {:.2})  {}",
                 t, hue, color.r, color.g, color.b, visual);
    }
}

/// Demo triadic color harmony
fn demo_triadic_harmony() {
    println!("\n=== Part 3: Triadic Color Harmony ===\n");

    let mut hsv1 = HsvToRgbOp::new();
    let mut hsv2 = HsvToRgbOp::new();
    let mut hsv3 = HsvToRgbOp::new();
    let ctx = EvalContext::new();

    println!("Triadic colors are 120 degrees apart on the color wheel.\n");

    let base_hues = [0.0, 30.0, 60.0, 90.0];

    for base_hue in base_hues {
        let hue1 = base_hue;
        let hue2 = (base_hue + 120.0) % 360.0;
        let hue3 = (base_hue + 240.0) % 360.0;

        // Set all to full saturation and value
        for (hsv, hue) in [(&mut hsv1, hue1), (&mut hsv2, hue2), (&mut hsv3, hue3)] {
            hsv.inputs_mut()[0].default = Value::Float(hue);
            hsv.inputs_mut()[1].default = Value::Float(1.0);
            hsv.inputs_mut()[2].default = Value::Float(1.0);
            hsv.inputs_mut()[3].default = Value::Float(1.0);
            hsv.compute(&ctx, &|_, _| Value::Float(0.0));
        }

        let c1 = hsv1.outputs()[0].value.as_color().unwrap();
        let c2 = hsv2.outputs()[0].value.as_color().unwrap();
        let c3 = hsv3.outputs()[0].value.as_color().unwrap();

        println!("  Base hue {:>3.0}: {} + {} + {}",
                 base_hue,
                 color_to_ascii(c1.r, c1.g, c1.b),
                 color_to_ascii(c2.r, c2.g, c2.b),
                 color_to_ascii(c3.r, c3.g, c3.b));
        println!("              Hues: {:.0}, {:.0}, {:.0}", hue1, hue2, hue3);
        println!();
    }
}

/// Demo color blending and adjustments
fn demo_color_blending() {
    println!("=== Part 4: Color Blending & Adjustments ===\n");

    let mut blend = BlendColorsOp::new();
    let mut adjust_brightness = AdjustBrightnessOp::new();
    let mut adjust_saturation = AdjustSaturationOp::new();
    let mut rgb_to_hsv = RgbToHsvOp::new();
    let ctx = EvalContext::new();

    // Create two colors to blend
    let mut hsv1 = HsvToRgbOp::new();
    let mut hsv2 = HsvToRgbOp::new();

    // Color A: Red
    hsv1.inputs_mut()[0].default = Value::Float(0.0);
    hsv1.inputs_mut()[1].default = Value::Float(1.0);
    hsv1.inputs_mut()[2].default = Value::Float(1.0);
    hsv1.inputs_mut()[3].default = Value::Float(1.0);
    hsv1.compute(&ctx, &|_, _| Value::Float(0.0));
    let color_a = hsv1.outputs()[0].value.as_color().unwrap();

    // Color B: Blue
    hsv2.inputs_mut()[0].default = Value::Float(240.0);
    hsv2.inputs_mut()[1].default = Value::Float(1.0);
    hsv2.inputs_mut()[2].default = Value::Float(1.0);
    hsv2.inputs_mut()[3].default = Value::Float(1.0);
    hsv2.compute(&ctx, &|_, _| Value::Float(0.0));
    let color_b = hsv2.outputs()[0].value.as_color().unwrap();

    println!("Blending Red {} to Blue {}:\n",
             color_to_ascii(color_a.r, color_a.g, color_a.b),
             color_to_ascii(color_b.r, color_b.g, color_b.b));

    blend.inputs_mut()[0].default = Value::Color(color_a);
    blend.inputs_mut()[1].default = Value::Color(color_b);

    println!("  {:>4}  {:>18}  Visual", "T", "RGB");
    println!("  {:->4}  {:->18}  {:->6}", "", "", "");

    for i in 0..=4 {
        let t = i as f32 * 0.25;
        blend.inputs_mut()[2].default = Value::Float(t);
        blend.compute(&ctx, &|_, _| Value::Float(0.0));
        let blended = blend.outputs()[0].value.as_color().unwrap();

        println!("  {:>4.2}  ({:.2}, {:.2}, {:.2})  {}",
                 t, blended.r, blended.g, blended.b,
                 color_to_ascii(blended.r, blended.g, blended.b));
    }

    // Brightness adjustment demo
    println!("\nBrightness adjustment on Orange {}:\n",
             color_to_ascii(1.0, 0.5, 0.0));

    // Start with orange
    hsv1.inputs_mut()[0].default = Value::Float(30.0);  // Orange hue
    hsv1.compute(&ctx, &|_, _| Value::Float(0.0));
    let orange = hsv1.outputs()[0].value.as_color().unwrap();

    adjust_brightness.inputs_mut()[0].default = Value::Color(orange);

    println!("  {:>8}  Visual", "Amount");
    println!("  {:->8}  {:->6}", "", "");

    for amount in [-0.4, -0.2, 0.0, 0.2, 0.4] {
        adjust_brightness.inputs_mut()[1].default = Value::Float(amount);
        adjust_brightness.compute(&ctx, &|_, _| Value::Float(0.0));
        let adjusted = adjust_brightness.outputs()[0].value.as_color().unwrap();

        println!("  {:>+8.1}  {}",
                 amount,
                 color_to_ascii(adjusted.r, adjusted.g, adjusted.b));
    }

    // Saturation adjustment demo
    println!("\nSaturation adjustment on Cyan {}:\n",
             color_to_ascii(0.0, 1.0, 1.0));

    // Start with cyan
    hsv1.inputs_mut()[0].default = Value::Float(180.0);  // Cyan hue
    hsv1.compute(&ctx, &|_, _| Value::Float(0.0));
    let cyan = hsv1.outputs()[0].value.as_color().unwrap();

    adjust_saturation.inputs_mut()[0].default = Value::Color(cyan);

    println!("  {:>8}  Visual  Description", "Amount");
    println!("  {:->8}  {:->6}  {:->12}", "", "", "");

    for (amount, desc) in [(-0.8, "Nearly gray"), (-0.4, "Muted"), (0.0, "Original")] {
        adjust_saturation.inputs_mut()[1].default = Value::Float(amount);
        adjust_saturation.compute(&ctx, &|_, _| Value::Float(0.0));
        let adjusted = adjust_saturation.outputs()[0].value.as_color().unwrap();

        println!("  {:>+8.1}  {}  {}",
                 amount,
                 color_to_ascii(adjusted.r, adjusted.g, adjusted.b),
                 desc);
    }

    // Round-trip demonstration
    println!("\nRGB -> HSV -> RGB round-trip verification:\n");

    let test_colors = [
        ("Red", 1.0, 0.0, 0.0),
        ("Green", 0.0, 1.0, 0.0),
        ("Orange", 1.0, 0.5, 0.0),
        ("Purple", 0.5, 0.0, 1.0),
    ];

    use flux_core::value::Color;

    for (name, r, g, b) in test_colors {
        let original = Color::rgba(r, g, b, 1.0);

        // RGB to HSV
        rgb_to_hsv.inputs_mut()[0].default = Value::Color(original);
        rgb_to_hsv.compute(&ctx, &|_, _| Value::Float(0.0));
        let h = rgb_to_hsv.outputs()[0].value.as_float().unwrap();
        let s = rgb_to_hsv.outputs()[1].value.as_float().unwrap();
        let v = rgb_to_hsv.outputs()[2].value.as_float().unwrap();

        // HSV back to RGB
        hsv1.inputs_mut()[0].default = Value::Float(h);
        hsv1.inputs_mut()[1].default = Value::Float(s);
        hsv1.inputs_mut()[2].default = Value::Float(v);
        hsv1.compute(&ctx, &|_, _| Value::Float(0.0));
        let converted = hsv1.outputs()[0].value.as_color().unwrap();

        let match_status = if (original.r - converted.r).abs() < 0.01 &&
                              (original.g - converted.g).abs() < 0.01 &&
                              (original.b - converted.b).abs() < 0.01 {
            "OK"
        } else {
            "MISMATCH"
        };

        println!("  {:>8}: HSV({:>5.1}, {:.2}, {:.2}) -> {} [{}]",
                 name, h, s, v,
                 color_to_ascii(converted.r, converted.g, converted.b),
                 match_status);
    }
}

/// Convert RGB to a simple ASCII color representation
fn color_to_ascii(r: f32, g: f32, b: f32) -> &'static str {
    // Determine dominant color channel(s)
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let range = max - min;

    if range < 0.1 {
        // Grayscale
        if max < 0.3 {
            return "[###]"; // Dark
        } else if max < 0.7 {
            return "[===]"; // Mid gray
        } else {
            return "[   ]"; // Light/white
        }
    }

    // Check for primary and secondary colors
    if r > 0.8 && g < 0.3 && b < 0.3 { return "[RED]"; }
    if g > 0.8 && r < 0.3 && b < 0.3 { return "[GRN]"; }
    if b > 0.8 && r < 0.3 && g < 0.3 { return "[BLU]"; }
    if r > 0.8 && g > 0.8 && b < 0.3 { return "[YLW]"; }
    if r > 0.8 && b > 0.8 && g < 0.3 { return "[MAG]"; }
    if g > 0.8 && b > 0.8 && r < 0.3 { return "[CYN]"; }

    // Orange variants
    if r > 0.8 && g > 0.3 && g < 0.7 && b < 0.3 { return "[ORG]"; }

    // Purple variants
    if r > 0.3 && r < 0.7 && b > 0.8 && g < 0.3 { return "[PRP]"; }

    // Mixed/other
    if r > g && r > b { return "[r--]"; }
    if g > r && g > b { return "[g--]"; }
    if b > r && b > g { return "[b--]"; }

    "[mix]"
}
