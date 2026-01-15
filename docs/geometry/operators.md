# Geometry Operators

Complete catalog of geometry operators organized by category.

## Overview

| Category | Count | Purpose |
|----------|-------|---------|
| Generators | 4 | Create geometry from parameters |
| Bridge | 5 | Extract/inject data from/to geometry |
| Modifiers | 3 | Transform existing geometry |
| GPU | 1 | Upload geometry for rendering |

## Generators

Create geometry from scratch based on input parameters.

### BoxGeometry

Creates an axis-aligned box mesh.

```
┌─────────────┐
│ BoxGeometry │
├─────────────┤
│ size: Vec3  │──▶ Geometry
│ center: Vec3│
│ divisions:  │
│   Vec3      │
└─────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `size` | Vec3 | [1, 1, 1] | Width, height, depth |
| `center` | Vec3 | [0, 0, 0] | Center position |
| `divisions` | Vec3 | [1, 1, 1] | Subdivisions per axis |

**Output:** `Geometry` — Box mesh with positions and computed normals

**Example:**
```rust
// Unit cube centered at origin
let op = BoxGeometry::new();

// 2x3x1 box offset from origin
let op = BoxGeometry::new()
    .with_size([2.0, 3.0, 1.0])
    .with_center([5.0, 0.0, 0.0]);
```

---

### SphereGeometry

Creates a UV sphere mesh.

```
┌────────────────┐
│ SphereGeometry │
├────────────────┤
│ radius: Float  │──▶ Geometry
│ center: Vec3   │
│ rows: Int      │
│ columns: Int   │
└────────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `radius` | Float | 1.0 | Sphere radius |
| `center` | Vec3 | [0, 0, 0] | Center position |
| `rows` | Int | 16 | Latitude divisions |
| `columns` | Int | 32 | Longitude divisions |

**Output:** `Geometry` — Sphere mesh with positions, normals, and UVs

**Generated attributes:**
- Point: `N` (normals pointing outward)
- Vertex: `uv` (spherical UV mapping)

---

### GridGeometry

Creates a flat grid mesh on the XZ plane.

```
┌──────────────┐
│ GridGeometry │
├──────────────┤
│ size: Vec2   │──▶ Geometry
│ center: Vec3 │
│ rows: Int    │
│ columns: Int │
└──────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `size` | Vec2 | [1, 1] | Width (X), depth (Z) |
| `center` | Vec3 | [0, 0, 0] | Center position |
| `rows` | Int | 10 | Divisions in Z |
| `columns` | Int | 10 | Divisions in X |

**Output:** `Geometry` — Grid mesh with positions, normals (Y-up), and UVs

---

### LineGeometry

Creates a line (polyline primitive) between two points.

```
┌──────────────┐
│ LineGeometry │
├──────────────┤
│ start: Vec3  │──▶ Geometry
│ end: Vec3    │
│ points: Int  │
└──────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `start` | Vec3 | [0, 0, 0] | Start point |
| `end` | Vec3 | [1, 0, 0] | End point |
| `points` | Int | 2 | Number of points along line |

**Output:** `Geometry` — Polyline primitive

---

## Bridge Operators

Extract data from geometry or inject data into geometry. These connect the geometry "bundle" to regular Flux types.

**Key design:** Bridge operators return `Arc`-wrapped data. Extraction is O(1), no copying. The returned `Value` shares data with the geometry.

### GeometryGetPositions

Extracts point positions as a Vec3List.

```
┌──────────────────────┐
│ GeometryGetPositions │
├──────────────────────┤
│ geometry: Geometry   │──▶ Vec3List
└──────────────────────┘
```

**Input:** `Geometry`
**Output:** `Vec3List` — Arc-wrapped reference to positions (O(1), zero copy)

**Performance:** Returns `ArcVec` that shares data with geometry. No allocation or copy.

---

### GeometryGetNormals

Extracts point normals as a Vec3List.

```
┌────────────────────┐
│ GeometryGetNormals │
├────────────────────┤
│ geometry: Geometry │──▶ Vec3List
└────────────────────┘
```

**Input:** `Geometry`
**Output:** `Vec3List` — Arc-wrapped normals (O(1), zero copy)

Returns empty list if geometry has no normal attribute.

---

### GeometryGetAttribute

Extracts any named attribute from geometry.

```
┌───────────────────────┐
│ GeometryGetAttribute  │
├───────────────────────┤
│ geometry: Geometry    │──▶ Value (list type)
│ name: String          │
│ level: AttributeLevel │
└───────────────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `geometry` | Geometry | — | Source geometry |
| `name` | String | — | Attribute name |
| `level` | AttributeLevel | Point | Which attribute level |

**Output:** `Value` — The attribute column (FloatList, Vec3List, etc.)

**AttributeLevel enum:**
```rust
pub enum AttributeLevel {
    Point,
    Vertex,
    Primitive,
    Detail,
}
```

---

### GeometrySetAttribute

Sets an attribute on geometry, returning modified geometry.

```
┌───────────────────────┐
│ GeometrySetAttribute  │
├───────────────────────┤
│ geometry: Geometry    │──▶ Geometry
│ name: String          │
│ level: AttributeLevel │
│ data: Value           │
└───────────────────────┘
```

**Inputs:**

| Port | Type | Description |
|------|------|-------------|
| `geometry` | Geometry | Source geometry |
| `name` | String | Attribute name to set |
| `level` | AttributeLevel | Which level to set on |
| `data` | Value (list) | Attribute data |

**Output:** `Geometry` — Geometry with attribute set

**Validation:** Data length must match element count at that level:
- Point: `data.len() == geometry.point_count()`
- Vertex: `data.len() == geometry.vertex_count()`
- Primitive: `data.len() == geometry.primitive_count()`
- Detail: `data.len() == 1`

---

### GeometryPointCount

Returns number of points in geometry.

```
┌─────────────────────┐
│ GeometryPointCount  │
├─────────────────────┤
│ geometry: Geometry  │──▶ Int
└─────────────────────┘
```

---

## Modifiers

Transform or combine existing geometry.

### GeometryTransform

Applies a transformation matrix to geometry.

```
┌───────────────────┐
│ GeometryTransform │
├───────────────────┤
│ geometry: Geometry│──▶ Geometry
│ matrix: Matrix4   │
└───────────────────┘
```

**Inputs:**

| Port | Type | Description |
|------|------|-------------|
| `geometry` | Geometry | Source geometry |
| `matrix` | Matrix4 | Transformation matrix |

**Output:** `Geometry` — Transformed geometry

**Transform rules:**
- Positions: Full transform (translate + rotate + scale)
- Normals: Inverse-transpose of upper-left 3×3 (preserves perpendicularity)
- Other Vec3 attributes: Rotate + scale only (no translation)

---

### GeometryMerge

Combines multiple geometries into one.

```
┌────────────────────┐
│ GeometryMerge      │
├────────────────────┤
│ geometry_a: Geometry│──▶ Geometry
│ geometry_b: Geometry│
└────────────────────┘
```

**Inputs:**

| Port | Type | Description |
|------|------|-------------|
| `geometry_a` | Geometry | First geometry |
| `geometry_b` | Geometry | Second geometry |

**Output:** `Geometry` — Combined geometry

Points, vertices, and primitives are concatenated. Attributes present in both geometries are concatenated; attributes in only one are padded with defaults.

**Future enhancement:** Accept list of geometries for merging N meshes.

---

### GeometrySubdivide (Future)

Subdivision surface smoothing.

```
┌───────────────────┐
│ GeometrySubdivide │
├───────────────────┤
│ geometry: Geometry│──▶ Geometry
│ iterations: Int   │
└───────────────────┘
```

Uses Catmull-Clark subdivision. Each iteration quadruples face count.

---

## GPU Operators

### UploadGeometry

Uploads CPU geometry to GPU buffers for rendering.

```
┌────────────────┐
│ UploadGeometry │
├────────────────┤
│ geometry:      │──▶ MeshHandle
│   Geometry     │
│ layout:        │
│   VertexLayout │
└────────────────┘
```

**Inputs:**

| Port | Type | Default | Description |
|------|------|---------|-------------|
| `geometry` | Geometry | — | CPU geometry data |
| `layout` | VertexLayout | Standard | Which attributes to upload |

**Output:** `MeshHandle` — GPU-resident mesh for rendering

**Behavior:**
1. Check geometry version against cached version
2. If unchanged, return cached MeshHandle (O(1))
3. If changed, build vertex buffer according to layout
4. Upload to GPU, cache handle and version
5. Return handle

### Version-Based Dirty Tracking

The operator caches the geometry's `version()` number:

```rust
pub struct UploadGeometry {
    cached_version: u64,
    cached_handle: Option<MeshHandle>,
}

impl Operator for UploadGeometry {
    fn evaluate(&mut self, inputs: &[Value], ctx: &EvalContext) -> Vec<Value> {
        let geometry = inputs[0].as_geometry().unwrap();

        // O(1) change detection via version number
        if Some(geometry.version()) == self.cached_version.map(|_| geometry.version())
           && self.cached_handle.is_some()
        {
            return vec![Value::MeshHandle(self.cached_handle.unwrap())];
        }

        // Geometry changed - re-upload
        let handle = ctx.gpu().upload_geometry(&geometry, &self.layout);
        self.cached_version = geometry.version();
        self.cached_handle = Some(handle);

        vec![Value::MeshHandle(handle)]
    }
}
```

### Vertex Layout Configuration

Instead of a fixed vertex layout, specify which attributes to include:

```rust
/// Configures which geometry attributes map to vertex buffer
pub struct VertexLayout {
    /// Attribute mappings: (geometry_attr_name, vertex_attr_location)
    pub attributes: Vec<VertexAttribute>,
}

pub struct VertexAttribute {
    /// Name in geometry (e.g., "N", "uv", "Cd")
    pub source: String,
    /// Shader location (e.g., 0=position, 1=normal, 2=uv)
    pub location: u32,
    /// Data type
    pub format: VertexFormat,
}

pub enum VertexFormat {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

impl VertexLayout {
    /// Standard PBR layout: position, normal, uv
    pub fn standard() -> Self {
        Self {
            attributes: vec![
                VertexAttribute { source: "P".into(), location: 0, format: VertexFormat::Vec3 },
                VertexAttribute { source: "N".into(), location: 1, format: VertexFormat::Vec3 },
                VertexAttribute { source: "uv".into(), location: 2, format: VertexFormat::Vec2 },
            ],
        }
    }

    /// Extended layout with vertex colors
    pub fn with_colors() -> Self {
        Self {
            attributes: vec![
                VertexAttribute { source: "P".into(), location: 0, format: VertexFormat::Vec3 },
                VertexAttribute { source: "N".into(), location: 1, format: VertexFormat::Vec3 },
                VertexAttribute { source: "uv".into(), location: 2, format: VertexFormat::Vec2 },
                VertexAttribute { source: "Cd".into(), location: 3, format: VertexFormat::Vec3 },
            ],
        }
    }

    /// Custom layout for specialized shaders
    pub fn custom(attributes: Vec<VertexAttribute>) -> Self {
        Self { attributes }
    }
}
```

### Handling Mixed Primitive Types

Geometry can contain polygons, polylines, and points. Upload handles this by:

1. **Separate draw calls:** Returns a MeshHandle that internally stores separate buffers per primitive type
2. **Filter parameter:** Optional input to upload only specific primitive types

```rust
pub enum PrimitiveFilter {
    All,                    // Upload everything
    Only(PrimitiveType),    // Only triangles, only lines, etc.
    Exclude(PrimitiveType), // Everything except points, etc.
}
```

### Missing Attributes

When a layout requests an attribute that doesn't exist in the geometry:

| Attribute | Default Value |
|-----------|---------------|
| `N` (normal) | `[0.0, 1.0, 0.0]` (Y-up) |
| `uv` | `[0.0, 0.0]` |
| `Cd` (color) | `[1.0, 1.0, 1.0]` (white) |
| Other | Error or skip based on `required` flag |

---

## Operator Patterns

### Generator Pattern

All generators follow a consistent pattern:

```rust
pub struct BoxGeometry {
    // Parameters with defaults
    size: [f32; 3],
    center: [f32; 3],
    divisions: [i32; 3],
}

impl Operator for BoxGeometry {
    fn inputs(&self) -> Vec<InputPort> {
        vec![
            InputPort::new("size", ValueType::Vec3),
            InputPort::new("center", ValueType::Vec3),
            InputPort::new("divisions", ValueType::Vec3),
        ]
    }

    fn outputs(&self) -> Vec<OutputPort> {
        vec![OutputPort::new("geometry", ValueType::Geometry)]
    }

    fn evaluate(&mut self, inputs: &[Value], ctx: &EvalContext) -> Vec<Value> {
        let size = inputs[0].as_vec3().unwrap_or(self.size);
        let center = inputs[1].as_vec3().unwrap_or(self.center);
        // ... generate geometry ...
        vec![Value::Geometry(Arc::new(geometry))]
    }
}
```

### Bridge Pattern

Bridge operators extract Arc-wrapped data (O(1), zero copy):

```rust
impl Operator for GeometryGetPositions {
    fn inputs(&self) -> Vec<InputPort> {
        vec![InputPort::new("geometry", ValueType::Geometry)]
    }

    fn outputs(&self) -> Vec<OutputPort> {
        vec![OutputPort::new("positions", ValueType::Vec3List)]
    }

    fn evaluate(&mut self, inputs: &[Value], _ctx: &EvalContext) -> Vec<Value> {
        let geometry = inputs[0].as_geometry().unwrap();
        // O(1) - just clone the Arc, not the data
        vec![Value::Vec3List(ArcVec(geometry.points().clone()))]
    }
}

impl Operator for GeometryGetAttribute {
    fn evaluate(&mut self, inputs: &[Value], _ctx: &EvalContext) -> Vec<Value> {
        let geometry = inputs[0].as_geometry().unwrap();
        let name = inputs[1].as_string().unwrap();
        let level = inputs[2].as_attribute_level().unwrap_or(AttributeLevel::Point);

        let table = match level {
            AttributeLevel::Point => geometry.point_attrs(),
            AttributeLevel::Vertex => geometry.vertex_attrs(),
            AttributeLevel::Primitive => geometry.prim_attrs(),
            AttributeLevel::Detail => {
                // Detail attrs are single values, not lists
                if let Some(value) = geometry.detail_attrs().get(name) {
                    return vec![value.clone()];
                }
                return vec![Value::default()];
            }
        };

        // O(1) - AttributeColumn::to_value() clones Arc, not data
        match table.get(name) {
            Some(column) => vec![column.to_value()],
            None => vec![Value::default()],
        }
    }
}
```

### Modifier Pattern

Modifiers use the mutation API which handles COW internally:

```rust
impl Operator for GeometryTransform {
    fn inputs(&self) -> Vec<InputPort> {
        vec![
            InputPort::new("geometry", ValueType::Geometry),
            InputPort::new("matrix", ValueType::Matrix4),
        ]
    }

    fn outputs(&self) -> Vec<OutputPort> {
        vec![OutputPort::new("geometry", ValueType::Geometry)]
    }

    fn evaluate(&mut self, inputs: &[Value], _ctx: &EvalContext) -> Vec<Value> {
        let geometry = inputs[0].as_geometry().unwrap();
        let matrix = inputs[1].as_matrix4().unwrap();

        // Clone the outer Arc (O(1))
        let mut result = Arc::clone(geometry);

        // Get mutable access - this handles COW internally
        // Only clones the points vector if it's shared
        let geo = Arc::make_mut(&mut result);
        geo.transform_positions(&matrix);

        // Transform normals with inverse-transpose if present
        if let Some(normals) = geo.point_attrs().get_vec3_arc("N") {
            let normal_matrix = matrix.inverse_transpose_3x3();
            let transformed: Vec<_> = normals
                .iter()
                .map(|n| normal_matrix.transform_vector(*n).normalize())
                .collect();
            geo.set_point_attr("N", Value::Vec3List(ArcVec::new(transformed)))
                .unwrap();
        }

        vec![Value::Geometry(result)]
    }
}
```

---

## Summary Table

| Operator | Category | Inputs | Output |
|----------|----------|--------|--------|
| BoxGeometry | Generator | size, center, divisions | Geometry |
| SphereGeometry | Generator | radius, center, rows, cols | Geometry |
| GridGeometry | Generator | size, center, rows, cols | Geometry |
| LineGeometry | Generator | start, end, points | Geometry |
| GeometryGetPositions | Bridge | geometry | Vec3List |
| GeometryGetNormals | Bridge | geometry | Vec3List |
| GeometryGetAttribute | Bridge | geometry, name, level | Value |
| GeometrySetAttribute | Bridge | geometry, name, level, data | Geometry |
| GeometryPointCount | Bridge | geometry | Int |
| GeometryTransform | Modifier | geometry, matrix | Geometry |
| GeometryMerge | Modifier | geometry_a, geometry_b | Geometry |
| UploadGeometry | GPU | geometry | MeshHandle |
