# flux-macros

Procedural macros for reducing operator implementation boilerplate.

## Features

- **`#[derive(Operator)]`** - Implements both `Operator` and `OperatorMeta` traits
- **`#[derive(OperatorMeta)]`** - Adds metadata to existing operators
- **Automatic getters/setters** - Generated from `#[input]` and `#[output]` fields
- **Rich metadata** - Labels, ranges, units, pin shapes

## Macros

| Macro | Description |
|-------|-------------|
| `#[derive(Operator)]` | Full implementation for new operators |
| `#[derive(OperatorMeta)]` | Metadata-only for existing operators |

## Quick Example

```rust
use flux_macros::Operator;
use flux_core::{Id, InputPort, OutputPort, EvalContext, InputResolver, OperatorMeta, Value};

#[derive(Operator)]
#[operator(name = "Divide", category = "Math", description = "Divides A by B")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
struct DivideOp {
    _id: Id,
    _inputs: Vec<InputPort>,
    _outputs: Vec<OutputPort>,
    #[input(label = "A", default = 0.0)]
    a: f32,
    #[input(label = "B", default = 1.0)]
    b: f32,
    #[output(label = "Result")]
    result: f32,
}

impl DivideOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);  // Generated getter
        let b = self.get_b(get_input);  // Generated getter
        self.set_result(if b != 0.0 { a / b } else { 0.0 });  // Generated setter
    }
}
```

## Struct-Level Attributes

### `#[operator(...)]`

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | String | Display name (defaults to struct name) |
| `category` | String | Category for grouping |
| `description` | String | Short description |
| `icon` | String | Optional icon identifier |
| `category_color` | [f32; 4] | RGBA color for UI |

## Field-Level Attributes

### `#[input(...)]`

| Attribute | Type | Description |
|-----------|------|-------------|
| `label` | String | Display label (defaults to field name) |
| `default` | literal | Default value |
| `range` | (f32, f32) | Min/max range for UI sliders |
| `unit` | String | Unit suffix (e.g., "px", "deg") |
| `shape` | String | Pin shape: "CircleFilled", "Square", etc. |

### `#[output(...)]`

| Attribute | Type | Description |
|-----------|------|-------------|
| `label` | String | Display label |
| `unit` | String | Unit suffix |
| `shape` | String | Pin shape (default: "TriangleFilled") |

## Generated Code

The `#[derive(Operator)]` macro generates:

1. **`new()` constructor** - Creates instance with default values
2. **`Default` impl** - Delegates to `new()`
3. **`Operator` trait impl** - `id()`, `name()`, `inputs()`, `outputs()`, `compute()`
4. **`OperatorMeta` trait impl** - `category()`, `description()`, `input_meta()`, `output_meta()`
5. **Getter methods** - `get_<field>(get_input)` for each input
6. **Setter methods** - `set_<field>(value)` for each output

## OperatorMeta-Only Derive

For operators that already implement `Operator`:

```rust
use flux_macros::OperatorMeta;

#[derive(OperatorMeta)]
#[meta(category = "Math", description = "Adds two numbers")]
#[meta(category_color = [0.35, 0.35, 0.55, 1.0])]
#[input_meta(0, label = "A")]
#[input_meta(1, label = "B", range = (0.0, 100.0), unit = "x")]
#[output_meta(0, label = "Sum", shape = "TriangleFilled")]
struct ExistingAddOp { /* ... */ }
```

## Supported Field Types

| Rust Type | Input Constructor | Output Constructor |
|-----------|------------------|-------------------|
| `f32` | `InputPort::float(...)` | `OutputPort::float(...)` |
| `i32` | `InputPort::int(...)` | `OutputPort::int(...)` |
| `bool` | `InputPort::bool(...)` | `OutputPort::bool(...)` |

## See Also

- [flux-core](../flux-core/) - Operator and OperatorMeta traits
- [flux-operators](../flux-operators/) - Examples of derived operators
- [Custom Operators](../docs/04-integration/custom-operators.md) - Building custom operators
