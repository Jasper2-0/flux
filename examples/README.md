# Flux Examples

29 examples organized into learning tiers for progressive mastery of Flux.

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
cargo run --example 07_diamond_dependency # Fan-out/fan-in patterns
cargo run --example 08_composite_operators # Encapsulation
cargo run --example 09_flow_control       # Conditionals and loops
cargo run --example 11_json_serialization # Save/load graphs
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

## Tier 2: Graph Patterns (07-10)

Essential patterns for building real applications.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 07 | `diamond_dependency` | Fan-out/fan-in, caching, topological evaluation |
| 08 | `composite_operators` | Subgraphs, encapsulation, expose I/O |
| 09 | `flow_control` | Switch, Gate, Loop, ForEach operators |
| 10 | `color_wheel` | HSV/RGB, color harmony, practical pipeline |

**Time to complete:** 1-2 hours

---

## Tier 3: Persistence & Registry (11-14)

Saving, loading, and dynamic operator creation.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 11 | `json_serialization` | SymbolDef, ChildDef, round-trip save/load |
| 12 | `enhanced_serialization` | Animation curves, keyframes, metadata |
| 13 | `operator_registry` | Dynamic creation, registry lookups |
| 14 | `symbol_instance` | Symbol definitions, instances, hierarchies |

**Time to complete:** 1-2 hours

---

## Tier 4: System Features (15-20)

Advanced capabilities and execution models.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 15 | `animation_system` | CurveBuilder, keyframes, loop modes |
| 16 | `dirty_flag_system` | Context-aware marking, lazy evaluation |
| 17 | `bypass_system` | Bypassing nodes, BypassState |
| 18 | `auto_conversion` | Type conversion nodes, ConversionOp |
| 19 | `trigger_system` | Push-based execution, trigger I/O |
| 20 | `playback_settings` | BPM timing, beat quantization |

**Time to complete:** 2-3 hours

---

## Tier 5: Performance (21-23)

Optimization, undo/redo, and benchmarking.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 21 | `compiled_execution` | Two-tier runtime, dead code elimination |
| 22 | `undo_redo` | Command pattern, MacroCommand, state management |
| 23 | `performance_benchmark` | Wide/deep graphs, interpreted vs compiled |

**Time to complete:** 1-2 hours

**Note:** Run benchmarks with `--release`:
```bash
cargo run --example 23_performance_benchmark --release
```

---

## Tier 6: Applications (24-28)

Real-world examples demonstrating complete solutions.

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 24 | `procedural_terrain` | Multi-octave FBM noise, terrain generation |
| 25 | `spring_physics` | Stateful operators, chained simulations |
| 26 | `state_machine` | Trigger-based FSM, edge detection |
| 27 | `list_processing` | List operators: map, filter, slice, concat |
| 28 | `collection_types` | Polymorphic lists, type-specific operators |

**Time to complete:** 2-3 hours

---

## Reference (29)

| # | Example | What You'll Learn |
|---|---------|-------------------|
| 29 | `operator_showcase` | All 150+ operators organized by category |

Use this as a reference catalog when you need to find a specific operator.

---

## Concept Index

Find examples by concept:

| Concept | Examples |
|---------|----------|
| **Graph basics** | 01, 02, 07, 08 |
| **Type system** | 04, 05, 06, 18, 28 |
| **Time/Animation** | 02, 15, 20 |
| **Serialization** | 11, 12, 14 |
| **Performance** | 16, 21, 23 |
| **Flow control** | 09, 19, 26 |
| **Collections** | 27, 28 |
| **Real-world apps** | 10, 24, 25, 26 |

---

## Tips

1. **Run examples in order** - Later examples build on earlier concepts
2. **Read the source** - Each example is heavily commented
3. **Experiment** - Modify values and add nodes to understand behavior
4. **Use `29_operator_showcase`** - Reference it when looking for specific operators
