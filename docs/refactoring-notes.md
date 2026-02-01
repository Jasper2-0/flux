# Flux Library Refactoring Notes

This document tracks medium-priority idiomaticity improvements identified during the Rust idiomaticity analysis (Feb 2026).

## Completed Refactors

- **TryFrom for Value** - Added `TryFrom<Value>` implementations for all types with `ValueTypeError`
- **Fixed macro type detection** - Replaced string-based type detection with proper `syn` AST matching
- **Newtype indices** - Added `InputIndex`, `OutputIndex`, `TriggerInputIndex`, `TriggerOutputIndex`
- **downcast-rs integration** - Replaced manual `as_any()` boilerplate with `impl_downcast!(Operator)`

---

## Remaining Medium Priority Refactors

### 1. Eliminate Marker Fields

**Location:** `flux-macros/src/lib.rs` (generated code)

**Current state:**
```rust
#[derive(Operator)]
struct TanOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Angle")]
    angle: f32,  // ← NEVER USED at runtime, wastes 4 bytes
    #[output(label = "Tan")]
    tan: f32,    // ← NEVER USED at runtime, wastes 4 bytes
}
```

**Issue:** Every operator with N inputs/outputs wastes N×4-8 bytes per instance. The fields exist only to carry attribute metadata at compile time.

**Proposed solutions:**

Option A - Use `PhantomData` (zero size):
```rust
#[input(label = "Angle")]
_angle: PhantomData<f32>,
```

Option B - Remove fields entirely, derive from attributes alone:
```rust
#[derive(Operator)]
#[input("Angle", f32, default = 0.0)]
#[output("Tan", f32)]
struct TanOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}
```

**Impact:** Low risk, moderate effort. Requires macro changes and updating all existing operators.

---

### 2. Replace `compute_impl` Convention

**Location:** All operators using derive macro

**Current state:**
```rust
impl TanOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        // User writes this - but the name is "magic" and not discoverable
    }
}
// Macro generates: fn compute(...) { self.compute_impl(ctx, get_input); }
```

**Issue:** "Magic method name" convention is not discoverable - IDEs won't help, and newcomers won't know to look for it.

**Proposed solutions:**

Option A - Trait-based:
```rust
trait ComputeOp {
    fn compute_op(&mut self, ctx: &EvalContext, get_input: InputResolver);
}

// User implements the trait, macro generates Operator::compute() delegation
```

Option B - Attribute macro:
```rust
#[flux::compute]
impl TanOp {
    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) { ... }
}
```

**Impact:** Medium risk, moderate effort. Changes the operator authoring API.

---

### 3. Support Polymorphic Operators in Macro

**Location:** `flux-macros/src/lib.rs`, `flux-operators/src/math/trig.rs`

**Current state:** `SinOp` and `CosOp` are manually implemented (~70 lines each) because they need `InputPort::arithmetic()` and `OutputPort::same_as_first()`:

```rust
// Must manually implement because macro doesn't support TypeConstraint
pub struct SinOp {
    id: Id,
    inputs: Vec<InputPort>,  // Uses Vec because port type varies
    outputs: Vec<OutputPort>,
}

impl SinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::arithmetic("Angle", Value::Float(0.0))],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }
}
// ... 50+ lines of Operator impl
// ... 20+ lines of OperatorMeta impl
```

**Proposed solution:**
```rust
#[derive(Operator)]
#[operator(name = "Sin", category = "Math")]
#[operator(polymorphic = "Arithmetic")]  // NEW: enables polymorphic ports
struct SinOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
    #[input(label = "Angle", constraint = "arithmetic")]
    angle: (),  // Type is determined at runtime
    #[output(label = "Sin", rule = "same_as_first")]
    result: (),
}
```

**Impact:** Medium risk, higher effort. Requires significant macro changes but would eliminate ~90 lines of boilerplate per polymorphic operator.

---

## Low Priority Items

- **Switch to `parking_lot`** - Replace `std::sync::RwLock` with `parking_lot::RwLock` in registry for marginal performance gains
- **Expand test coverage** - Add more integration tests for edge cases

---

## References

- Original analysis: `.claude/projects/.../6b1d5871-e579-4725-bf8b-5e70d520117a.jsonl`
- Related files:
  - `libs/flux/flux-macros/src/lib.rs`
  - `libs/flux/flux-core/src/operator.rs`
  - `libs/flux/flux-operators/src/math/trig.rs`
