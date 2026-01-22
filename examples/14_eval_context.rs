//! Demo 14: Full Evaluation Context
//!
//! This example demonstrates the complete EvalContext features:
//! - Time and frame management
//! - Camera setup with projection/view matrices
//! - Fog and PBR material parameters
//! - Point light management
//! - Context variables (float, int, bool, string, object)
//! - Gizmo visibility settings
//! - Child contexts (local time, FX time)
//! - Call contexts for subroutine/loop cache isolation
//!
//! Run with: `cargo run --example 14_eval_context`

use flux_core::{
    CallContext, Camera, EvalContext, FogParameters, GizmoVisibility, PbrMaterial,
    PerspectiveCamera, PointLight, TransformGizmoMode, Value,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 14: Full Evaluation Context       ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a full context
    let mut ctx = EvalContext::new();
    println!("Initial context:");
    println!("  Time: {:.3}s, Frame: {}", ctx.time, ctx.frame);
    println!("  Resolution: {:?}", ctx.resolution);
    println!("  Delta time: {:.4}s", ctx.delta_time);

    // Advance time
    ctx.advance(0.016); // ~60fps
    ctx.advance(0.016);
    ctx.advance(0.016);
    println!("\nAfter 3 frames (48ms):");
    println!("  Time: {:.3}s, Frame: {}", ctx.time, ctx.frame);
    println!("  Delta time: {:.4}s", ctx.delta_time);

    // Set up camera
    let camera = PerspectiveCamera::look_at(
        [0.0, 5.0, 10.0], // position
        [0.0, 0.0, 0.0],  // target
        [0.0, 1.0, 0.0],  // up
    );
    ctx.set_camera(&camera);
    println!("\nCamera setup:");
    println!("  Position: {:?}", camera.get_position());
    println!(
        "  View matrix set: {}",
        ctx.world_to_camera
            != [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
    );
    println!(
        "  Projection matrix set: {}",
        ctx.camera_to_clip
            != [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
    );

    // Set up fog
    ctx.fog = FogParameters::linear(5.0, 50.0, [0.7, 0.7, 0.8, 1.0]);
    println!("\nFog settings:");
    println!("  Enabled: {}", ctx.fog.enabled);
    println!("  Range: {} to {}", ctx.fog.start, ctx.fog.end);
    println!("  Color: {:?}", ctx.fog.color);

    // Set up PBR material
    ctx.pbr_material = PbrMaterial::metal([0.8, 0.6, 0.2, 1.0], 0.3);
    println!("\nPBR Material (Gold):");
    println!("  Albedo: {:?}", ctx.pbr_material.albedo);
    println!("  Metallic: {}", ctx.pbr_material.metallic);
    println!("  Roughness: {}", ctx.pbr_material.roughness);

    // Add lights
    ctx.add_point_light(PointLight::new([5.0, 5.0, 5.0], [1.0, 1.0, 1.0], 2.0));
    ctx.add_point_light(PointLight::new([-5.0, 3.0, 0.0], [1.0, 0.5, 0.3], 1.5));
    println!("\nLighting:");
    println!("  Point lights: {}", ctx.point_lights.len());
    for (i, light) in ctx.point_lights.iter().enumerate() {
        println!(
            "    Light {}: pos={:?}, color={:?}, intensity={}",
            i, light.position, light.color, light.intensity
        );
    }

    // Context variables
    ctx.set_float_var("speed", 2.5);
    ctx.set_float_var("amplitude", 1.0);
    ctx.set_int_var("iterations", 100);
    ctx.set_bool_var("debug_mode", true);
    ctx.set_string_var("layer_name", "main");
    ctx.set_object_var("custom", Value::Vec3([1.0, 2.0, 3.0]));

    println!("\nContext variables:");
    println!("  speed (float): {:?}", ctx.get_float_var("speed"));
    println!("  amplitude (float): {:?}", ctx.get_float_var("amplitude"));
    println!("  iterations (int): {:?}", ctx.get_int_var("iterations"));
    println!("  debug_mode (bool): {:?}", ctx.get_bool_var("debug_mode"));
    println!("  layer_name (string): {:?}", ctx.get_string_var("layer_name"));
    println!("  custom (object): {:?}", ctx.get_object_var("custom"));

    // Get with defaults
    println!("\nVariables with defaults:");
    println!("  missing_float: {}", ctx.get_float_var_or("missing", 0.0));
    println!("  missing_int: {}", ctx.get_int_var_or("missing", -1));
    println!(
        "  missing_string: {}",
        ctx.get_string_var_or("missing", "default")
    );

    // Gizmos
    ctx.show_gizmos = GizmoVisibility::IfSelected;
    ctx.transform_gizmo_mode = TransformGizmoMode::Move;
    println!("\nGizmo settings:");
    println!("  Visibility: {:?}", ctx.show_gizmos);
    println!("  Transform mode: {:?}", ctx.transform_gizmo_mode);
    println!(
        "  Should show (selected=true): {}",
        ctx.should_show_gizmos(true)
    );
    println!(
        "  Should show (selected=false): {}",
        ctx.should_show_gizmos(false)
    );

    // Local time context
    let child_ctx = ctx.with_local_time(100.0);
    println!("\nChild context with local time:");
    println!("  Parent time: {:.3}s", ctx.time);
    println!("  Child local time: {:.1}s", child_ctx.local_time);
    println!("  Child still has lights: {}", child_ctx.point_lights.len());

    // FX time context
    let fx_ctx = ctx.with_fx_time(42.0);
    println!("\nFX context:");
    println!("  Local FX time: {:.1}s", fx_ctx.local_fx_time);
    println!("  Time changed (res=0.01): {}", ctx.has_time_changed(0.01));

    // Reset
    ctx.reset();
    println!("\nAfter reset:");
    println!("  Time: {:.3}s, Frame: {}", ctx.time, ctx.frame);
    println!("  Variables cleared: {}", ctx.float_vars.is_empty());
    println!("  Lights cleared: {}", ctx.point_lights.is_empty());

    // =========================================================================
    // Call Context for Subroutine/Loop Cache Isolation
    // =========================================================================
    println!("\n╔════════════════════════════════════════╗");
    println!("║ Call Context (Cache Isolation)         ║");
    println!("╚════════════════════════════════════════╝\n");

    // CallContext is used when evaluating graphs with subroutines or loops.
    // It ensures that the same operator evaluated in different contexts
    // (e.g., different loop iterations) gets separate cache entries.

    // Create a fresh context
    let ctx = EvalContext::new();
    println!("Root call context: {:?}", ctx.call_context);

    // Simulate a loop with 3 iterations - each gets a unique call context
    println!("\nLoop iterations (simulated):");
    for i in 0..3 {
        let iter_ctx = ctx.with_call_context(i);
        println!(
            "  Iteration {}: call_context = {:?}",
            i, iter_ctx.call_context
        );
    }

    // Nested loops create hierarchical contexts
    println!("\nNested loops:");
    for outer in 0..2 {
        let outer_ctx = ctx.with_call_context(outer);
        println!(
            "  Outer {}: call_context = {:?}",
            outer, outer_ctx.call_context
        );
        for inner in 0..2 {
            let inner_ctx = outer_ctx.with_call_context(inner);
            println!(
                "    Inner {}: call_context = {:?}",
                inner, inner_ctx.call_context
            );
        }
    }

    // Demonstrate direct CallContext usage
    println!("\nDirect CallContext API:");
    let root = CallContext::root();
    println!("  Root: raw={}", root.raw());

    let child_0 = root.child(0);
    let child_1 = root.child(1);
    println!("  Child 0: raw={}", child_0.raw());
    println!("  Child 1: raw={}", child_1.raw());
    println!("  Children are different: {}", child_0 != child_1);

    // Nested children
    let nested = child_0.child(0);
    println!("  Nested (child_0.child(0)): raw={}", nested.raw());
    println!(
        "  Nested differs from parent's sibling: {}",
        nested != child_1.child(0)
    );

    println!("\nPurpose of CallContext:");
    println!("  - When a subroutine is called multiple times, shared operators");
    println!("    would return cached values from the wrong invocation.");
    println!("  - CallContext ensures each invocation gets separate cache entries.");
    println!("  - This is critical for correct behavior of loops and recursion.");
}
