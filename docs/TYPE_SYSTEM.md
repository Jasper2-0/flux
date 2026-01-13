# Flux Type System Specification

> **Version**: 1.0 Draft
> **Status**: Design Document
> **Last Updated**: 2025-01

This document defines the type system for Flux, a node-based visual programming system. It establishes the rules for value types, type categories, coercion, broadcasting, and polymorphic operators.

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Value Types](#value-types)
3. [Type Categories](#type-categories)
4. [Type Coercion](#type-coercion)
5. [Broadcasting Rules](#broadcasting-rules)
6. [Polymorphic Operators](#polymorphic-operators)
7. [Error Handling](#error-handling)
8. [Future Types](#future-types)
9. [Implementation Notes](#implementation-notes)

---

## Design Principles

### Core Philosophy

1. **Permissive Connections**: Any output can connect to any input. Invalid type combinations fall back to default values silently. This prioritizes creative flow over strict type safety.

2. **Wider Type Wins**: When mixing types in arithmetic, the result is the "wider" type. `Float + Vec3 = Vec3` (the scalar broadcasts to match the vector).

3. **Context-Dependent Promotion**: Integer arithmetic preserves integers (`Int + Int = Int`), but mixing with floats promotes to float (`Int + Float = Float`).

4. **Semantic Bridges Over Implicit Coercion**: Complex types (Mesh, Texture, Curve) do not implicitly convert. Explicit bridge operators extract data when needed.

5. **Sensible Defaults**: Every type has a meaningful default value used when coercion fails or inputs are disconnected.

### Non-Goals

- Strict type checking that prevents connections
- Compile-time type errors
- Complex generic/template type parameters

---

## Value Types

Flux supports **18 value types** organized into families.

### Primitives

| Type | Rust Type | Default | Description |
|------|-----------|---------|-------------|
| `Float` | `f32` | `0.0` | 32-bit floating point |
| `Int` | `i32` | `0` | 32-bit signed integer |
| `Bool` | `bool` | `false` | Boolean true/false |

### Vectors

| Type | Rust Type | Default | Description |
|------|-----------|---------|-------------|
| `Vec2` | `[f32; 2]` | `[0.0, 0.0]` | 2D vector (x, y) |
| `Vec3` | `[f32; 3]` | `[0.0, 0.0, 0.0]` | 3D vector (x, y, z) |
| `Vec4` | `[f32; 4]` | `[0.0, 0.0, 0.0, 0.0]` | 4D vector (x, y, z, w) |

### Text

| Type | Rust Type | Default | Description |
|------|-----------|---------|-------------|
| `String` | `String` | `""` | UTF-8 text |

### Complex Types

| Type | Rust Type | Default | Description |
|------|-----------|---------|-------------|
| `Color` | `Color { r, g, b, a }` | White `(1,1,1,1)` | RGBA color, components 0.0-1.0 |
| `Gradient` | `Gradient { stops }` | Black→White | Color gradient with stops |
| `Matrix4` | `[[f32; 4]; 4]` | Identity | 4x4 transformation matrix |

### Collections

| Type | Rust Type | Default | Description |
|------|-----------|---------|-------------|
| `FloatList` | `Vec<f32>` | `[]` | List of floats |
| `IntList` | `Vec<i32>` | `[]` | List of integers |
| `BoolList` | `Vec<bool>` | `[]` | List of booleans |
| `Vec2List` | `Vec<[f32; 2]>` | `[]` | List of 2D vectors |
| `Vec3List` | `Vec<[f32; 3]>` | `[]` | List of 3D vectors |
| `Vec4List` | `Vec<[f32; 4]>` | `[]` | List of 4D vectors |
| `ColorList` | `Vec<Color>` | `[]` | List of colors |
| `StringList` | `Vec<String>` | `[]` | List of strings |

---

## Type Categories

Type categories group related types for polymorphic operations. A type can belong to multiple categories.

### Category Definitions

| Category | Member Types | Use Case |
|----------|--------------|----------|
| `Numeric` | Float, Int | Scalar arithmetic |
| `Vector` | Vec2, Vec3, Vec4 | Vector operations |
| `ColorLike` | Color, Vec3, Vec4 | Color manipulation |
| `List` | All `*List` types | Collection operations |
| `Matrix` | Matrix4 | Transform operations |
| `Arithmetic` | Float, Int, Vec2, Vec3, Vec4, Color | Types supporting +, -, *, / |
| `Any` | All types | Unconstrained inputs |

### Category Membership Matrix

```
              Numeric  Vector  ColorLike  List  Matrix  Arithmetic
Float            ✓                                          ✓
Int              ✓                                          ✓
Bool
Vec2                      ✓                                 ✓
Vec3                      ✓       ✓                         ✓
Vec4                      ✓       ✓                         ✓
String
Color                             ✓                         ✓
Gradient
Matrix4                                    ✓
FloatList                                  ✓
IntList                                    ✓
BoolList                                   ✓
Vec2List                                   ✓
Vec3List                                   ✓
Vec4List                                   ✓
ColorList                                  ✓
StringList                                 ✓
```

---

## Type Coercion

Coercion is the automatic conversion of a value from one type to another. Coercion happens:
- At connection time (via ConversionOp insertion)
- At compute time (via fallback extraction)

### Coercion Rules

#### Numeric Coercions (Bidirectional)

```
Int ↔ Float     (cast)
Bool → Int      (false=0, true=1)
Bool → Float    (false=0.0, true=1.0)
Int → Bool      (0=false, else=true)
Float → Bool    (0.0=false, else=true)
```

#### Vector/Color Coercions

```
Vec4 ↔ Color           (isomorphic: [r,g,b,a] ↔ Color{r,g,b,a})
Vec3 → Vec4            (add w=1.0)
Vec4 → Vec3            (drop w)
Vec3 → Color           (add a=1.0)
Color → Vec3           (drop a)
```

#### Scalar Broadcasting

```
Float → Vec2           ([f, f])
Float → Vec3           ([f, f, f])
Float → Vec4           ([f, f, f, f])
Float → Color          (rgba(f, f, f, 1.0) — grayscale)
```

#### String Coercions

```
Int → String           (to_string)
Float → String         (to_string)
Bool → String          ("true" / "false")
```

#### Scalar → List Wrapping

```
Float → FloatList      ([f])
Int → IntList          ([i])
Bool → BoolList        ([b])
Vec2 → Vec2List        ([v])
Vec3 → Vec3List        ([v])
Vec4 → Vec4List        ([v])
Color → ColorList      ([c])
String → StringList    ([s])
```

#### List ↔ List Coercions

```
IntList ↔ FloatList          (element-wise cast)
ColorList ↔ Vec4List         (element-wise isomorphic)
Vec2List → FloatList         (flatten: [x,y, x,y, ...])
Vec3List → FloatList         (flatten: [x,y,z, x,y,z, ...])
Vec4List → FloatList         (flatten: [x,y,z,w, x,y,z,w, ...])
FloatList → Vec2List         (group by 2, truncate remainder)
FloatList → Vec3List         (group by 3, truncate remainder)
FloatList → Vec4List         (group by 4, truncate remainder)
```

### Non-Coercible Types

The following types have **no coercion paths** (except identity):

- `Gradient` — Must use sampling operators
- `Matrix4` — Must use decomposition operators
- `String` — Cannot convert TO string types (only FROM primitives)

---

## Broadcasting Rules

Broadcasting applies when performing arithmetic between values of different "widths".

### Width Hierarchy

```
Float < Vec2 < Vec3 < Vec4
  ↓      ↓      ↓      ↓
Int    (n/a)  Color  Color
```

### Broadcasting Semantics

When operands have different widths, the narrower value broadcasts to match the wider:

| Operation | Result Type | Semantics |
|-----------|-------------|-----------|
| `Float + Vec2` | `Vec2` | `[f+x, f+y]` |
| `Float + Vec3` | `Vec3` | `[f+x, f+y, f+z]` |
| `Float + Vec4` | `Vec4` | `[f+x, f+y, f+z, f+w]` |
| `Float + Color` | `Color` | `rgba(f+r, f+g, f+b, a)` — alpha preserved |
| `Float * Vec3` | `Vec3` | `[f*x, f*y, f*z]` |
| `Vec3 + Vec3` | `Vec3` | `[x1+x2, y1+y2, z1+z2]` |
| `Int + Float` | `Float` | Int promotes to Float first |
| `Int + Vec3` | `Vec3` | Int promotes to Float, then broadcasts |

### Color-Specific Rules

- Scalar + Color: Scalar applies to RGB, alpha preserved
- Color + Color: Component-wise RGBA addition
- Scalar * Color: Scalar applies to RGBA (including alpha)

### Integer Preservation

```
Int + Int = Int
Int - Int = Int
Int * Int = Int
Int / Int = Int    (truncated division)
Int % Int = Int

Int + Float = Float   (promotion)
Float + Int = Float   (promotion)
```

---

## Polymorphic Operators

Polymorphic operators accept multiple input types and adapt their behavior accordingly.

### Type Constraint System

Input ports declare constraints rather than fixed types:

```rust
// Old: Fixed type
InputPort::float("A", 0.0)

// New: Constrained type
InputPort::arithmetic("A", Value::Float(0.0))  // Accepts Float, Int, Vec2, Vec3, Vec4, Color
```

### Output Type Inference

Output types are derived from input types:

| Rule | Description | Example |
|------|-------------|---------|
| `Fixed(T)` | Always produces type T | `Sin` always outputs Float |
| `SameAsInput(n)` | Matches input n's type | `Add` output matches wider input |
| `Wider(a, b)` | Wider of inputs a and b | `Add(Float, Vec3)` → Vec3 |

### Type Resolution Flow

1. User connects wire from source to target
2. Graph checks if source type satisfies target constraint
3. If yes: connection established, output types propagate
4. If no: connection established anyway (permissive), fallback at runtime

### Fallback Behavior

When types don't match at compute time:

```rust
// Trying to add String + Vec3
let a: String = "hello";
let b: Vec3 = [1.0, 2.0, 3.0];

// String has no arithmetic interpretation
// Result: Vec3 default [0.0, 0.0, 0.0] + [1.0, 2.0, 3.0] = [1.0, 2.0, 3.0]
```

The incompatible operand falls back to the default value for the expected type.

---

## Error Handling

### Philosophy: Permissive with Sensible Defaults

Flux prioritizes creative flow over strict correctness. The system never blocks a connection or crashes due to type mismatches.

### Fallback Values

When coercion fails, use the type's default:

| Target Type | Fallback Value |
|-------------|----------------|
| Float | `0.0` |
| Int | `0` |
| Bool | `false` |
| Vec2 | `[0.0, 0.0]` |
| Vec3 | `[0.0, 0.0, 0.0]` |
| Vec4 | `[0.0, 0.0, 0.0, 0.0]` |
| Color | White `(1, 1, 1, 1)` |
| String | `""` |
| Any List | `[]` |

### Debugging Support

While errors don't prevent execution, operators can log warnings:

```rust
// In compute():
if !a.is_arithmetic() {
    log::warn!("Add: input A ({:?}) is not arithmetic, using default", a.value_type());
}
```

Future: Visual indicator on nodes with type warnings (non-blocking).

---

## Future Types

As Flux grows, new types will be added. This section establishes guidelines.

### Planned Types

| Type | Category | Coercions | Notes |
|------|----------|-----------|-------|
| `Mesh` | Geometry | None (isolated) | Vertices, faces, normals |
| `Curve` | Geometry | None (isolated) | Bezier/NURBS curves |
| `Texture` | Image | None (isolated) | 2D image data |
| `Audio` | Signal | None (isolated) | Audio buffer |

### Semantic Bridge Pattern

Complex types don't coerce implicitly. Instead, explicit operators extract data:

```
Mesh → Vec3List        via MeshGetVertices operator
Mesh → IntList         via MeshGetFaceIndices operator
Texture → Color        via SampleTexture(uv) operator
Texture → ColorList    via TextureToPixels operator
Curve → Vec3List       via SampleCurve(count) operator
```

**Rationale**: Implicit coercion hides intent. `MeshGetVertices` makes it clear the user wants vertex positions, not normals or UVs.

### Adding New Types Checklist

1. Add variant to `Value` enum
2. Add variant to `ValueType` enum
3. Implement `default_value()` for the type
4. Add to appropriate `TypeCategory` memberships
5. Add `as_*` accessor method on `Value`
6. Add convenience constructors to `InputPort` and `OutputPort`
7. Implement `Serialize`/`Deserialize`
8. Create semantic bridge operators if data extraction is needed
9. Update this specification document

---

## Implementation Notes

### Files to Modify for Type System Changes

```
flux-core/src/value/mod.rs       # Value enum, ValueType, coercion
flux-core/src/value/color.rs     # Color type implementation
flux-core/src/value/gradient.rs  # Gradient type implementation
flux-core/src/value/matrix.rs    # Matrix4 type implementation
flux-core/src/port/input.rs      # InputPort, type constraints
flux-core/src/port/output.rs     # OutputPort, type inference
flux-core/src/error.rs           # Type-related errors
```

### Implementing `std::ops` for Value

Arithmetic operations should be implemented on `Value` directly:

```rust
impl std::ops::Add for Value {
    type Output = Option<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a + b)),
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a + b)),
            (Value::Vec3(a), Value::Vec3(b)) => Some(Value::Vec3([a[0]+b[0], a[1]+b[1], a[2]+b[2]])),
            // Broadcasting
            (Value::Float(f), Value::Vec3(v)) => Some(Value::Vec3([f+v[0], f+v[1], f+v[2]])),
            (Value::Vec3(v), Value::Float(f)) => Some(Value::Vec3([v[0]+f, v[1]+f, v[2]+f])),
            // ... other combinations
            _ => None, // Fallback handled by caller
        }
    }
}
```

### Type Constraint Enum

```rust
pub enum TypeConstraint {
    /// Exactly this type
    Exact(ValueType),

    /// Any type in this category
    Category(TypeCategory),

    /// Any of these specific types
    OneOf(Vec<ValueType>),

    /// Output type matches this input's resolved type
    SameAsInput(usize),

    /// Accept any type
    Any,
}
```

---

## Appendix: Complete Coercion Matrix

✓ = direct coercion supported
○ = coercion via intermediate type
· = no coercion (use default)

```
FROM →        Float Int Bool Vec2 Vec3 Vec4 String Color Grad Mat4 FList IList BList V2List V3List V4List CList SList
TO ↓
Float           ✓    ✓    ✓    ·    ·    ·    ·      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Int             ✓    ✓    ✓    ·    ·    ·    ·      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Bool            ✓    ✓    ✓    ·    ·    ·    ·      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Vec2            ✓    ○    ·    ✓    ·    ·    ·      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Vec3            ✓    ○    ·    ·    ✓    ✓    ·      ✓    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Vec4            ✓    ○    ·    ·    ✓    ✓    ·      ✓    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
String          ✓    ✓    ✓    ·    ·    ·    ✓      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Color           ✓    ○    ·    ·    ✓    ✓    ·      ✓    ·    ·     ·     ·     ·      ·      ·      ·     ·     ·
Gradient        ·    ·    ·    ·    ·    ·    ·      ·    ✓    ·     ·     ·     ·      ·      ·      ·     ·     ·
Matrix4         ·    ·    ·    ·    ·    ·    ·      ·    ·    ✓     ·     ·     ·      ·      ·      ·     ·     ·
FloatList       ✓    ○    ·    ·    ·    ·    ·      ·    ·    ·     ✓     ✓     ·      ✓      ✓      ✓     ·     ·
IntList         ○    ✓    ·    ·    ·    ·    ·      ·    ·    ·     ✓     ✓     ·      ·      ·      ·     ·     ·
BoolList        ·    ·    ✓    ·    ·    ·    ·      ·    ·    ·     ·     ·     ✓      ·      ·      ·     ·     ·
Vec2List        ·    ·    ·    ✓    ·    ·    ·      ·    ·    ·     ✓     ·     ·      ✓      ·      ·     ·     ·
Vec3List        ·    ·    ·    ·    ✓    ·    ·      ·    ·    ·     ✓     ·     ·      ·      ✓      ·     ·     ·
Vec4List        ·    ·    ·    ·    ·    ✓    ·      ·    ·    ·     ✓     ·     ·      ·      ·      ✓     ✓     ·
ColorList       ·    ·    ·    ·    ·    ·    ·      ✓    ·    ·     ·     ·     ·      ·      ·      ✓     ✓     ·
StringList      ·    ·    ·    ·    ·    ·    ✓      ·    ·    ·     ·     ·     ·      ·      ·      ·     ·     ✓
```

---

*This document is the authoritative reference for Flux type system behavior. All implementations should conform to these specifications.*
