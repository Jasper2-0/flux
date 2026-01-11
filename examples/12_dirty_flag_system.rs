//! Demo 12: Enhanced Dirty Flag System
//!
//! This example demonstrates the dirty flag system for lazy evaluation:
//! - Different trigger types (None, Always, Animated, TimeChanged, FrameChanged)
//! - Context-aware dirty checking
//! - Global invalidation control
//! - Explicit dirty marking
//!
//! Run with: `cargo run --example 12_dirty_flag_system`

use flux_core::{
    advance_invalidation_frame, reset_invalidation_frame, DirtyFlag, DirtyFlagTrigger, EvalContext,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 12: Enhanced Dirty Flag System    ║");
    println!("╚════════════════════════════════════════╝\n");

    reset_invalidation_frame();

    // Create flags with different triggers
    let mut flag_none = DirtyFlag::with_trigger(DirtyFlagTrigger::None);
    let mut flag_always = DirtyFlag::with_trigger(DirtyFlagTrigger::Always);
    let mut flag_animated = DirtyFlag::with_trigger(DirtyFlagTrigger::Animated);
    let mut flag_time = DirtyFlag::with_trigger(DirtyFlagTrigger::TimeChanged);
    let mut flag_frame = DirtyFlag::with_trigger(DirtyFlagTrigger::FrameChanged);

    let mut ctx = EvalContext::new();

    // Clean all flags
    flag_none.mark_clean_for_context(&ctx);
    flag_always.mark_clean_for_context(&ctx);
    flag_animated.mark_clean_for_context(&ctx);
    flag_time.mark_clean_for_context(&ctx);
    flag_frame.mark_clean_for_context(&ctx);

    println!("After cleaning all flags (t=0, frame=0):");
    println!(
        "  None trigger:     dirty={}",
        flag_none.is_dirty_for_context(&ctx)
    );
    println!(
        "  Always trigger:   dirty={}",
        flag_always.is_dirty_for_context(&ctx)
    );
    println!(
        "  Animated trigger: dirty={}",
        flag_animated.is_dirty_for_context(&ctx)
    );
    println!(
        "  Time trigger:     dirty={}",
        flag_time.is_dirty_for_context(&ctx)
    );
    println!(
        "  Frame trigger:    dirty={}",
        flag_frame.is_dirty_for_context(&ctx)
    );

    // Advance context
    ctx.advance(0.016); // ~60fps
    println!("\nAfter advancing context (t=0.016, frame=1):");
    println!(
        "  None trigger:     dirty={}",
        flag_none.is_dirty_for_context(&ctx)
    );
    println!(
        "  Always trigger:   dirty={}",
        flag_always.is_dirty_for_context(&ctx)
    );
    println!(
        "  Animated trigger: dirty={}",
        flag_animated.is_dirty_for_context(&ctx)
    );
    println!(
        "  Time trigger:     dirty={}",
        flag_time.is_dirty_for_context(&ctx)
    );
    println!(
        "  Frame trigger:    dirty={}",
        flag_frame.is_dirty_for_context(&ctx)
    );

    // Global invalidation
    advance_invalidation_frame();
    println!("\nAfter global invalidation:");
    println!(
        "  Animated trigger: dirty={}",
        flag_animated.is_dirty_for_context(&ctx)
    );

    // Explicit mark dirty
    flag_none.mark_dirty();
    println!("\nAfter marking None flag dirty:");
    println!(
        "  None trigger:     dirty={}",
        flag_none.is_dirty_for_context(&ctx)
    );
}
