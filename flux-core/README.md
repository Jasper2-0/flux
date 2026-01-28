# flux-core

Foundation types and traits for the Flux dataflow system.

## Features

- **Type-safe values** - 18 value variants with automatic coercion
- **Lazy evaluation** - DirtyFlag system for efficient recomputation
- **Extensible context** - EvalContext with timing, camera, and custom extensions
- **Port definitions** - InputPort/OutputPort for operator connections

## Key Types

| Type | Description |
|------|-------------|
| `Value` | Sum type with 18 variants (Float, Vec3, Color, Gradient, etc.) |
| `ValueType` | Type identifiers for runtime type checking |
| `Operator` | Core trait that all operators implement |
| `EvalContext` | Evaluation context with timing, camera, materials |
| `InputPort` / `OutputPort` | Port definitions for connecting operators |
| `DirtyFlag` | Lazy evaluation tracking for caching |
| `Id` | UUID-based unique identifier for nodes |

## Value Variants

```
Primitives:  Float, Int, Bool
Vectors:     Vec2, Vec3, Vec4
Complex:     String, Color, Gradient, Matrix4
Collections: FloatList, IntList, BoolList, Vec2List, Vec3List, Vec4List, ColorList, StringList
```

## Quick Example

```rust
use flux_core::{Value, EvalContext, InputPort, OutputPort};

// Create typed values
let position = Value::Vec3([1.0, 2.0, 3.0]);
let color = Value::Color(Color::rgba(1.0, 0.5, 0.0, 1.0));

// Type coercion is automatic
let as_vec4 = position.coerce_to(ValueType::Vec4); // [1.0, 2.0, 3.0, 1.0]

// Create evaluation context with timing
let mut ctx = EvalContext::new();
ctx.advance(0.016); // ~60fps frame

// Define ports for operators
let input = InputPort::float("amplitude", 1.0);
let output = OutputPort::vec3("position");
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        flux-core                             │
│  ┌─────────┐  ┌─────────────┐  ┌───────────────────────┐   │
│  │   Id    │  │   Value     │  │     EvalContext       │   │
│  │ (UUID)  │  │ (type-safe) │  │ (timing, camera, etc) │   │
│  └─────────┘  └─────────────┘  └───────────────────────┘   │
│  ┌─────────────────────────┐   ┌───────────────────────┐   │
│  │  InputPort / OutputPort │   │      Operator         │   │
│  │   (connection points)   │   │   (trait definition)  │   │
│  └─────────────────────────┘   └───────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| `value` | Value enum, ValueType, Color, Gradient, Matrix4 |
| `context` | EvalContext, Camera, PbrMaterial, PointLight |
| `operator` | Operator trait, InputResolver |
| `port` | InputPort, OutputPort, TriggerInput/Output |
| `dirty_flag` | DirtyFlag, DirtyFlagSet for lazy evaluation |
| `operator_meta` | OperatorMeta trait, PortMeta, PinShape |
| `error` | OperatorError, EvalResult types |

## See Also

- [flux-operators](../flux-operators/) - 110+ operator implementations
- [flux-graph](../flux-graph/) - Graph execution engine
- [Type System docs](../docs/01-concepts/type-system.md) - Value types and coercion rules
