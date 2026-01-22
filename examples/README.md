# Flux Examples

32 examples organized into learning tiers for progressive mastery of Flux.

## Quick Start

```bash
# Start with the basics
cargo run --example 01_basic_arithmetic

# Follow the numbered sequence for structured learning
cargo run --example 02_sine_wave
cargo run --example 03_multi_input_sum
# ... and so on
```

## Learning Paths

### Path A: Quick Start (2-3 hours)
Get up and running with core concepts:

```bash
cargo run --example 01_basic_arithmetic   # Graph basics
cargo run --example 02_sine_wave          # Time-based values
cargo run --example 03_multi_input_sum    # Variadic inputs
cargo run --example 04_compare_operator   # Boolean logic
cargo run --example 05_vec3_composition   # Vector types
cargo run --example 06_type_validation    # Type system
```

### Path B: Production Ready (6-8 hours)
Add patterns and persistence:

```bash
# After Path A, continue with:
cargo run --example 07_json_serialization # Save/load graphs
cargo run --example 08_composite_operators # Encapsulation
cargo run --example 23_flow_control       # Conditionals and loops
cargo run --example 24_diamond_dependency # Fan-out/fan-in patterns
cargo run --example 21_compiled_execution # Performance optimization
```

### Path C: Full Mastery (15-20 hours)
Complete all examples in order (01-29).

---

## Tier 1: Foundation (01-06)

Core graph concepts every user should understand.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 01 | `basic_arithmetic` | Graph construction, operators, connections, lazy evaluation |
| 02 | `sine_wave` | Time context, oscillators, frequency/amplitude |
| 03 | `multi_input_sum` | Variadic inputs, multi-input ports |
| 04 | `compare_operator` | Boolean output, comparison operators |
| 05 | `vec3_composition` | Vector types, component animation |
| 06 | `type_validation` | Type safety, error handling |

**Time to complete:** 1-2 hours

---

## Tier 2: Persistence & Registry (07-09)

Saving, loading, and dynamic operator creation.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 07 | `json_serialization` | SymbolDef, ChildDef, round-trip save/load |
| 08 | `composite_operators` | Subgraphs, encapsulation, expose I/O |
| 09 | `operator_registry` | Dynamic creation, registry lookups |

**Time to complete:** 1-2 hours

---

## Tier 3: Animation & Symbols (10-13)

Animation systems and symbol hierarchies.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 10 | `animation_system` | CurveBuilder, keyframes, loop modes |
| 11 | `symbol_instance` | Symbol definitions, instances, hierarchies |
| 12 | `dirty_flag_system` | Context-aware marking, lazy evaluation |
| 13 | `bypass_system` | Bypassing nodes, BypassState |

**Time to complete:** 1-2 hours

---

## Tier 4: Context & Settings (14-18)

Evaluation contexts and advanced features.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 14 | `eval_context` | Evaluation context, custom contexts |
| 15 | `playback_settings` | BPM timing, beat quantization |
| 16 | `enhanced_serialization` | Animation curves, keyframes, metadata |
| 17 | `resource_management` | Resource handling patterns |
| 18 | `phase3_operators` | Advanced operator categories |

**Time to complete:** 2-3 hours

---

## Tier 5: Execution & Flow (19-23)

Triggers, compilation, and flow control.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 19 | `auto_conversion` | Type conversion nodes, ConversionOp |
| 20 | `trigger_system` | Push-based execution, trigger I/O |
| 21 | `compiled_execution` | Two-tier runtime, dead code elimination |
| 22 | `undo_redo` | Command pattern, MacroCommand, state management |
| 23 | `flow_control` | Switch, Gate, Loop, ForEach operators |

**Time to complete:** 2-3 hours

---

## Tier 6: Applications (24-29)

Real-world examples demonstrating complete solutions.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 24 | `diamond_dependency` | Fan-out/fan-in, caching, topological evaluation |
| 25 | `color_wheel` | HSV/RGB, color harmony, practical pipeline |
| 26 | `performance_benchmark` | Wide/deep graphs, interpreted vs compiled |
| 27 | `procedural_terrain` | Multi-octave FBM noise, terrain generation |
| 28 | `spring_physics` | Stateful operators, chained simulations |
| 29 | `state_machine` | Trigger-based FSM, edge detection |

**Time to complete:** 2-3 hours

**Note:** Run benchmarks with `--release`:
```bash
cargo run --example 26_performance_benchmark --release
```

---

## Tier 7: Collections & Reference (30-32)

Advanced list operations and comprehensive operator reference.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 30 | `list_processing` | List operators: map, filter, slice, concat |
| 31 | `collection_types` | Polymorphic lists, type-specific operators |
| 32 | `operator_showcase` | All 150+ operators organized by category |

**Time to complete:** 1-2 hours

---

## Concept Index

Find examples by concept:

| Concept | Examples |
|---------|----------|
| **Graph basics** | 01, 02, 08, 24 |
| **Type system** | 04, 05, 06, 19, 31 |
| **Time/Animation** | 02, 10, 15 |
| **Serialization** | 07, 11, 16 |
| **Performance** | 12, 21, 26 |
| **Flow control** | 20, 23, 29 |
| **Collections** | 30, 31 |
| **Real-world apps** | 25, 27, 28, 29 |
| **Reference** | 32 |

---

## Tips

1. **Run examples in order** - Later examples build on earlier concepts
2. **Read the source** - Each example is heavily commented
3. **Experiment** - Modify values and add nodes to understand behavior
