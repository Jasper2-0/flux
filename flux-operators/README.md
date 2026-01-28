# flux-operators

110+ built-in operator implementations for the Flux graph system.

## Features

- **Organized by category** - Math, vector, color, time, flow, and more
- **Registry system** - Dynamic operator creation by name or type ID
- **Derive macro support** - Re-exports `#[derive(Operator)]` from flux-macros
- **Comprehensive coverage** - From basic arithmetic to complex flow control

## Operator Categories

| Category | Count | Description |
|----------|-------|-------------|
| `builtin` | 10 | Core operators (Constant, Add, Multiply, SineWave) |
| `math` | 35 | Arithmetic, trig, interpolation, noise |
| `logic` | 12 | Boolean operations, integer math |
| `vector` | 15 | Vec2/Vec3/Vec4 operations |
| `color` | 8 | Color manipulation, gradients |
| `time` | 10 | Clocks, oscillators, accumulators |
| `flow` | 12 | State, conditionals, context variables |
| `string` | 8 | String manipulation |
| `list` | 8 | List operations |
| `util` | 6 | Debug, passthrough, type conversion |

## Quick Example

```rust
use flux_operators::{create_default_registry, OperatorRegistry};
use flux_core::EvalContext;

// Create registry with all built-in operators
let registry = create_default_registry();

// Create operators by name
let add = registry.create("Add").unwrap();
let sine = registry.create("SineWave").unwrap();

// List available operators
for name in registry.operator_names() {
    println!("{}", name);
}
```

## Using the Derive Macro

```rust
use flux_operators::Operator;
use flux_core::{Id, InputPort, OutputPort, EvalContext, InputResolver, Value};

#[derive(Operator)]
#[operator(name = "Scale", category = "Math", description = "Multiplies input by factor")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
struct ScaleOp {
    _id: Id,
    _inputs: Vec<InputPort>,
    _outputs: Vec<OutputPort>,
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[input(label = "Factor", default = 1.0, range = (0.0, 10.0))]
    factor: f32,
    #[output(label = "Result")]
    result: f32,
}

impl ScaleOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let factor = self.get_factor(get_input);
        self.set_result(value * factor);
    }
}
```

## Registry API

| Method | Description |
|--------|-------------|
| `create(name)` | Create operator by name |
| `create_with_params(name, params)` | Create with initial parameters |
| `operator_names()` | List all registered operator names |
| `get_meta(name)` | Get operator metadata without creating |
| `register(name, factory)` | Register a custom operator |

## Key Operators by Category

### Math
`Add`, `Subtract`, `Multiply`, `Divide`, `Sin`, `Cos`, `Lerp`, `Clamp`, `Remap`, `PerlinNoise`

### Vector
`Vec3Compose`, `Vec3Decompose`, `Normalize`, `Dot`, `Cross`, `Distance`, `Reflect`

### Color
`RgbaColor`, `HsvToRgb`, `RgbToHsv`, `BlendColors`, `SampleGradient`

### Time
`Time`, `DeltaTime`, `SineWave`, `SawWave`, `Spring`, `Accumulator`

### Flow
`Switch`, `Gate`, `Counter`, `Delay`, `GetFloatVar`, `SetFloatVar`

## See Also

- [flux-core](../flux-core/) - Foundation types (Value, Operator trait)
- [flux-macros](../flux-macros/) - Procedural macros for operator implementation
- [flux-gpu](../flux-gpu/) - GPU-accelerated operators
- [Operator Catalog](../docs/05-reference/operator-catalog.md) - Complete operator reference
