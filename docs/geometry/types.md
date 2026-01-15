# Geometry Types

Core type definitions for the Flux geometry system.

## Type Hierarchy

```
Value::Geometry(Arc<Geometry>)
         │
         ▼
    ┌─────────────────────────────────────────────────────┐
    │ Geometry                                             │
    │   version: u64                     ◄── dirty tracking│
    │   points: Arc<Vec<[f32; 3]>>       ◄── positions     │
    │   vertices: Arc<Vec<u32>>          ◄── topology      │
    │   primitive_starts: Arc<Vec<u32>>                    │
    │   primitive_types: Arc<Vec<PrimitiveType>>           │
    │                                                      │
    │   point_attrs: Arc<AttributeTable> ◄── per-point     │
    │   vertex_attrs: Arc<AttributeTable>◄── per-corner    │
    │   prim_attrs: Arc<AttributeTable>  ◄── per-face      │
    │   detail_attrs: Arc<DetailAttrs>   ◄── global        │
    └─────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Private fields, validated mutation

All fields are private. Mutation goes through methods that maintain invariants:
- `primitive_starts.len() == primitive_types.len() + 1`
- `primitive_starts.last() == vertices.len()`
- All `point_attrs` columns have length == `points.len()`
- All `vertex_attrs` columns have length == `vertices.len()`

### 2. Fine-grained copy-on-write

Internal vectors are `Arc`-wrapped. Modifying just positions doesn't clone normals:
```rust
let positions = self.points_mut();  // Only clones points if shared
```

### 3. Explicit version tracking

Every mutation increments `version`. Downstream operators (like GPU upload) compare versions to detect changes in O(1).

### 4. Thread safety

`Geometry` is `Send + Sync`. Safe to share across threads, upload from background thread, etc.

---

## Geometry

The primary geometry container. Stores topology and attributes with enforced invariants.

```rust
use std::sync::Arc;

/// A geometry container with topology and per-element attributes.
///
/// All fields are private to enforce invariants. Use the provided methods
/// for construction and mutation.
///
/// # Thread Safety
///
/// `Geometry` is `Send + Sync`. Internal Arc-wrapped data can be safely
/// shared across threads.
///
/// # Copy-on-Write
///
/// Internal data is Arc-wrapped for fine-grained COW. Cloning a Geometry
/// is O(1). Mutation methods use `Arc::make_mut` to clone only the
/// specific component being modified.
#[derive(Clone, Debug)]
pub struct Geometry {
    /// Monotonically increasing version number. Incremented on every mutation.
    /// Used by downstream operators to detect changes without content comparison.
    version: u64,

    // ═══════════════════════════════════════════════════════════════
    // TOPOLOGY (Arc-wrapped for fine-grained COW)
    // ═══════════════════════════════════════════════════════════════

    /// Point positions
    points: Arc<Vec<[f32; 3]>>,

    /// Vertex-to-point indices
    vertices: Arc<Vec<u32>>,

    /// Start index in `vertices` for each primitive (CSR format)
    /// Length: num_primitives + 1. Last entry equals vertices.len()
    primitive_starts: Arc<Vec<u32>>,

    /// Type of each primitive
    primitive_types: Arc<Vec<PrimitiveType>>,

    // ═══════════════════════════════════════════════════════════════
    // ATTRIBUTES (Arc-wrapped for fine-grained COW)
    // ═══════════════════════════════════════════════════════════════

    /// Per-point attributes. Each column length == points.len()
    point_attrs: Arc<AttributeTable>,

    /// Per-vertex attributes. Each column length == vertices.len()
    vertex_attrs: Arc<AttributeTable>,

    /// Per-primitive attributes. Each column length == primitive_count()
    prim_attrs: Arc<AttributeTable>,

    /// Detail (whole-geometry) attributes. Single values, not lists.
    detail_attrs: Arc<DetailAttrs>,
}

/// Primitive topology type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// Closed polygon (triangle, quad, n-gon)
    Polygon,
    /// Open polyline
    Polyline,
    /// Single point (no connectivity)
    Point,
}

// Ensure thread safety
static_assertions::assert_impl_all!(Geometry: Send, Sync);
```

---

## Construction

### Empty geometry

```rust
impl Geometry {
    /// Create empty geometry with version 0
    pub fn new() -> Self {
        Self {
            version: 0,
            points: Arc::new(Vec::new()),
            vertices: Arc::new(Vec::new()),
            primitive_starts: Arc::new(vec![0]),
            primitive_types: Arc::new(Vec::new()),
            point_attrs: Arc::new(AttributeTable::new()),
            vertex_attrs: Arc::new(AttributeTable::new()),
            prim_attrs: Arc::new(AttributeTable::new()),
            detail_attrs: Arc::new(DetailAttrs::new()),
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new()
    }
}
```

### Builder pattern

For constructing geometry with validation:

```rust
/// Builder for constructing valid Geometry
pub struct GeometryBuilder {
    points: Vec<[f32; 3]>,
    indices: Vec<u32>,        // Flat list of vertex indices
    counts: Vec<u32>,         // Vertex count per primitive
    types: Vec<PrimitiveType>,
    point_attrs: AttributeTable,
    vertex_attrs: AttributeTable,
    prim_attrs: AttributeTable,
    detail_attrs: DetailAttrs,
}

impl GeometryBuilder {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
            counts: Vec::new(),
            types: Vec::new(),
            point_attrs: AttributeTable::new(),
            vertex_attrs: AttributeTable::new(),
            prim_attrs: AttributeTable::new(),
            detail_attrs: DetailAttrs::new(),
        }
    }

    /// Add a point, returns its index
    pub fn add_point(&mut self, position: [f32; 3]) -> u32 {
        let idx = self.points.len() as u32;
        self.points.push(position);
        idx
    }

    /// Add a triangle primitive
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) -> &mut Self {
        self.indices.extend([a, b, c]);
        self.counts.push(3);
        self.types.push(PrimitiveType::Polygon);
        self
    }

    /// Add a quad primitive
    pub fn add_quad(&mut self, a: u32, b: u32, c: u32, d: u32) -> &mut Self {
        self.indices.extend([a, b, c, d]);
        self.counts.push(4);
        self.types.push(PrimitiveType::Polygon);
        self
    }

    /// Add a polyline primitive
    pub fn add_polyline(&mut self, point_indices: &[u32]) -> &mut Self {
        self.indices.extend(point_indices);
        self.counts.push(point_indices.len() as u32);
        self.types.push(PrimitiveType::Polyline);
        self
    }

    /// Set point attribute (must match point count when building)
    pub fn set_point_attr(&mut self, name: impl Into<String>, data: impl Into<Value>) -> &mut Self {
        // Validation happens in build()
        self.point_attrs.set_unchecked(name.into(), data.into());
        self
    }

    /// Set vertex attribute (must match vertex count when building)
    pub fn set_vertex_attr(&mut self, name: impl Into<String>, data: impl Into<Value>) -> &mut Self {
        self.vertex_attrs.set_unchecked(name.into(), data.into());
        self
    }

    /// Build the geometry, validating all invariants
    pub fn build(self) -> Result<Geometry, GeometryError> {
        // Validate point attributes
        for (name, value) in self.point_attrs.iter() {
            let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
                name: name.clone(),
                expected: "list type",
                got: value.value_type(),
            })?;
            if len != self.points.len() {
                return Err(GeometryError::LengthMismatch {
                    context: format!("point attribute '{}'", name),
                    expected: self.points.len(),
                    got: len,
                });
            }
        }

        // Validate vertex attributes
        for (name, value) in self.vertex_attrs.iter() {
            let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
                name: name.clone(),
                expected: "list type",
                got: value.value_type(),
            })?;
            if len != self.indices.len() {
                return Err(GeometryError::LengthMismatch {
                    context: format!("vertex attribute '{}'", name),
                    expected: self.indices.len(),
                    got: len,
                });
            }
        }

        // Validate primitive attributes
        for (name, value) in self.prim_attrs.iter() {
            let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
                name: name.clone(),
                expected: "list type",
                got: value.value_type(),
            })?;
            if len != self.types.len() {
                return Err(GeometryError::LengthMismatch {
                    context: format!("primitive attribute '{}'", name),
                    expected: self.types.len(),
                    got: len,
                });
            }
        }

        // Validate all indices are in bounds
        for &idx in &self.indices {
            if idx as usize >= self.points.len() {
                return Err(GeometryError::InvalidPointIndex {
                    index: idx as usize,
                    max: self.points.len(),
                });
            }
        }

        // Build primitive_starts from counts
        let mut primitive_starts = Vec::with_capacity(self.counts.len() + 1);
        primitive_starts.push(0);
        let mut offset = 0u32;
        for count in &self.counts {
            offset += count;
            primitive_starts.push(offset);
        }

        Ok(Geometry {
            version: 0,
            points: Arc::new(self.points),
            vertices: Arc::new(self.indices),
            primitive_starts: Arc::new(primitive_starts),
            primitive_types: Arc::new(self.types),
            point_attrs: Arc::new(self.point_attrs),
            vertex_attrs: Arc::new(self.vertex_attrs),
            prim_attrs: Arc::new(self.prim_attrs),
            detail_attrs: Arc::new(self.detail_attrs),
        })
    }
}
```

---

## Read Access

Immutable access returns references or Arc clones (O(1)):

```rust
impl Geometry {
    // ═══════════════════════════════════════════════════════════════
    // VERSION & IDENTITY
    // ═══════════════════════════════════════════════════════════════

    /// Get current version number. Increments on any mutation.
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    // ═══════════════════════════════════════════════════════════════
    // COUNTS
    // ═══════════════════════════════════════════════════════════════

    #[inline]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    #[inline]
    pub fn primitive_count(&self) -> usize {
        self.primitive_types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    // ═══════════════════════════════════════════════════════════════
    // TOPOLOGY ACCESS (returns Arc for zero-cost sharing)
    // ═══════════════════════════════════════════════════════════════

    /// Get positions as shared Arc (O(1), no copy)
    #[inline]
    pub fn points(&self) -> &Arc<Vec<[f32; 3]>> {
        &self.points
    }

    /// Get vertices as shared Arc (O(1), no copy)
    #[inline]
    pub fn vertices(&self) -> &Arc<Vec<u32>> {
        &self.vertices
    }

    /// Get primitive types as shared Arc
    #[inline]
    pub fn primitive_types(&self) -> &Arc<Vec<PrimitiveType>> {
        &self.primitive_types
    }

    // ═══════════════════════════════════════════════════════════════
    // PRIMITIVE ITERATION
    // ═══════════════════════════════════════════════════════════════

    /// Iterate over primitives
    pub fn primitives(&self) -> PrimitiveIter<'_> {
        PrimitiveIter {
            geometry: self,
            index: 0,
        }
    }

    /// Get a specific primitive
    pub fn primitive(&self, index: usize) -> Option<Primitive<'_>> {
        if index >= self.primitive_count() {
            return None;
        }
        Some(Primitive {
            geometry: self,
            index,
        })
    }

    // ═══════════════════════════════════════════════════════════════
    // ATTRIBUTE ACCESS (returns Arc for zero-cost sharing)
    // ═══════════════════════════════════════════════════════════════

    /// Get point attributes table
    #[inline]
    pub fn point_attrs(&self) -> &Arc<AttributeTable> {
        &self.point_attrs
    }

    /// Get vertex attributes table
    #[inline]
    pub fn vertex_attrs(&self) -> &Arc<AttributeTable> {
        &self.vertex_attrs
    }

    /// Get primitive attributes table
    #[inline]
    pub fn prim_attrs(&self) -> &Arc<AttributeTable> {
        &self.prim_attrs
    }

    /// Get detail attributes
    #[inline]
    pub fn detail_attrs(&self) -> &Arc<DetailAttrs> {
        &self.detail_attrs
    }

    // ═══════════════════════════════════════════════════════════════
    // CONVENIENCE ACCESSORS
    // ═══════════════════════════════════════════════════════════════

    /// Get point normals if present (returns Arc, O(1))
    pub fn normals(&self) -> Option<Arc<Vec<[f32; 3]>>> {
        self.point_attrs.get_vec3_arc("N")
    }

    /// Get vertex UVs if present (returns Arc, O(1))
    pub fn uvs(&self) -> Option<Arc<Vec<[f32; 2]>>> {
        self.vertex_attrs.get_vec2_arc("uv")
    }

    /// Compute axis-aligned bounding box
    pub fn bounds(&self) -> Option<Aabb> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = self.points[0];
        let mut max = self.points[0];
        for p in self.points.iter().skip(1) {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        Some(Aabb { min, max })
    }
}

/// Axis-aligned bounding box
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
```

---

## Primitive Iteration

```rust
/// Iterator over primitives
pub struct PrimitiveIter<'a> {
    geometry: &'a Geometry,
    index: usize,
}

impl<'a> Iterator for PrimitiveIter<'a> {
    type Item = Primitive<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.geometry.primitive_count() {
            return None;
        }
        let prim = Primitive {
            geometry: self.geometry,
            index: self.index,
        };
        self.index += 1;
        Some(prim)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.geometry.primitive_count() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for PrimitiveIter<'_> {}

/// A view into a single primitive
#[derive(Clone, Copy)]
pub struct Primitive<'a> {
    geometry: &'a Geometry,
    index: usize,
}

impl<'a> Primitive<'a> {
    /// Primitive type (polygon, polyline, point)
    pub fn primitive_type(&self) -> PrimitiveType {
        self.geometry.primitive_types[self.index]
    }

    /// Vertex indices for this primitive
    pub fn vertex_indices(&self) -> &'a [u32] {
        let start = self.geometry.primitive_starts[self.index] as usize;
        let end = self.geometry.primitive_starts[self.index + 1] as usize;
        &self.geometry.vertices[start..end]
    }

    /// Number of vertices in this primitive
    pub fn vertex_count(&self) -> usize {
        let start = self.geometry.primitive_starts[self.index] as usize;
        let end = self.geometry.primitive_starts[self.index + 1] as usize;
        end - start
    }

    /// Get positions for this primitive's vertices
    pub fn positions(&self) -> impl Iterator<Item = [f32; 3]> + 'a {
        self.vertex_indices()
            .iter()
            .map(|&idx| self.geometry.points[idx as usize])
    }

    /// Is this a triangle?
    pub fn is_triangle(&self) -> bool {
        self.primitive_type() == PrimitiveType::Polygon && self.vertex_count() == 3
    }

    /// Is this a quad?
    pub fn is_quad(&self) -> bool {
        self.primitive_type() == PrimitiveType::Polygon && self.vertex_count() == 4
    }
}
```

---

## Mutation

Mutation methods use `Arc::make_mut` for copy-on-write and increment version:

```rust
impl Geometry {
    /// Increment version. Called by all mutation methods.
    #[inline]
    fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    // ═══════════════════════════════════════════════════════════════
    // POSITION MUTATION
    // ═══════════════════════════════════════════════════════════════

    /// Get mutable access to positions (COW - only clones if shared)
    pub fn points_mut(&mut self) -> &mut Vec<[f32; 3]> {
        self.bump_version();
        Arc::make_mut(&mut self.points)
    }

    /// Transform all positions by a matrix
    pub fn transform_positions(&mut self, matrix: &Matrix4) {
        let points = self.points_mut();
        for p in points.iter_mut() {
            *p = matrix.transform_point(*p);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // ATTRIBUTE MUTATION
    // ═══════════════════════════════════════════════════════════════

    /// Set a point attribute. Validates length matches point count.
    pub fn set_point_attr(
        &mut self,
        name: impl Into<String>,
        value: Value,
    ) -> Result<(), GeometryError> {
        let name = name.into();
        let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
            name: name.clone(),
            expected: "list type",
            got: value.value_type(),
        })?;

        if len != self.point_count() {
            return Err(GeometryError::LengthMismatch {
                context: format!("point attribute '{}'", name),
                expected: self.point_count(),
                got: len,
            });
        }

        self.bump_version();
        Arc::make_mut(&mut self.point_attrs).set_unchecked(name, value);
        Ok(())
    }

    /// Set a vertex attribute. Validates length matches vertex count.
    pub fn set_vertex_attr(
        &mut self,
        name: impl Into<String>,
        value: Value,
    ) -> Result<(), GeometryError> {
        let name = name.into();
        let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
            name: name.clone(),
            expected: "list type",
            got: value.value_type(),
        })?;

        if len != self.vertex_count() {
            return Err(GeometryError::LengthMismatch {
                context: format!("vertex attribute '{}'", name),
                expected: self.vertex_count(),
                got: len,
            });
        }

        self.bump_version();
        Arc::make_mut(&mut self.vertex_attrs).set_unchecked(name, value);
        Ok(())
    }

    /// Set a primitive attribute. Validates length matches primitive count.
    pub fn set_prim_attr(
        &mut self,
        name: impl Into<String>,
        value: Value,
    ) -> Result<(), GeometryError> {
        let name = name.into();
        let len = value.list_len().ok_or(GeometryError::InvalidAttributeType {
            name: name.clone(),
            expected: "list type",
            got: value.value_type(),
        })?;

        if len != self.primitive_count() {
            return Err(GeometryError::LengthMismatch {
                context: format!("primitive attribute '{}'", name),
                expected: self.primitive_count(),
                got: len,
            });
        }

        self.bump_version();
        Arc::make_mut(&mut self.prim_attrs).set_unchecked(name, value);
        Ok(())
    }

    /// Set a detail attribute (single value for whole geometry)
    pub fn set_detail_attr(&mut self, name: impl Into<String>, value: Value) {
        self.bump_version();
        Arc::make_mut(&mut self.detail_attrs).set(name.into(), value);
    }

    /// Get mutable access to point attributes (COW)
    pub fn point_attrs_mut(&mut self) -> &mut AttributeTable {
        self.bump_version();
        Arc::make_mut(&mut self.point_attrs)
    }

    // ═══════════════════════════════════════════════════════════════
    // GEOMETRY OPERATIONS
    // ═══════════════════════════════════════════════════════════════

    /// Merge another geometry into this one
    pub fn merge(&mut self, other: &Geometry) {
        self.bump_version();

        let point_offset = self.point_count() as u32;
        let vertex_offset = self.vertex_count() as u32;

        // Merge points
        Arc::make_mut(&mut self.points).extend(other.points.iter());

        // Merge vertices with offset
        Arc::make_mut(&mut self.vertices)
            .extend(other.vertices.iter().map(|&v| v + point_offset));

        // Merge primitive starts with offset
        let prim_starts = Arc::make_mut(&mut self.primitive_starts);
        for &start in other.primitive_starts.iter().skip(1) {
            prim_starts.push(start + vertex_offset);
        }

        // Merge primitive types
        Arc::make_mut(&mut self.primitive_types).extend(other.primitive_types.iter());

        // Merge attributes (extend columns, pad missing with defaults)
        Arc::make_mut(&mut self.point_attrs)
            .merge_extend(&other.point_attrs, self.point_count() - other.point_count(), other.point_count());
        Arc::make_mut(&mut self.vertex_attrs)
            .merge_extend(&other.vertex_attrs, self.vertex_count() - other.vertex_count(), other.vertex_count());
        Arc::make_mut(&mut self.prim_attrs)
            .merge_extend(&other.prim_attrs, self.primitive_count() - other.primitive_count(), other.primitive_count());
    }
}
```

---

## AttributeTable

Stores named attribute columns with Arc-wrapped data for zero-cost extraction:

```rust
/// A table of named attribute columns.
///
/// Each column stores data as Arc-wrapped vectors, enabling O(1) extraction
/// for bridge operators.
#[derive(Clone, Debug, Default)]
pub struct AttributeTable {
    // Using Arc<str> for keys to avoid string allocation on lookup
    columns: HashMap<Arc<str>, AttributeColumn>,
}

/// A single attribute column with Arc-wrapped data
#[derive(Clone, Debug)]
pub enum AttributeColumn {
    Float(Arc<Vec<f32>>),
    Int(Arc<Vec<i32>>),
    Vec2(Arc<Vec<[f32; 2]>>),
    Vec3(Arc<Vec<[f32; 3]>>),
    Vec4(Arc<Vec<[f32; 4]>>),
}

impl AttributeColumn {
    pub fn len(&self) -> usize {
        match self {
            Self::Float(v) => v.len(),
            Self::Int(v) => v.len(),
            Self::Vec2(v) => v.len(),
            Self::Vec3(v) => v.len(),
            Self::Vec4(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Float(_) => ValueType::FloatList,
            Self::Int(_) => ValueType::IntList,
            Self::Vec2(_) => ValueType::Vec2List,
            Self::Vec3(_) => ValueType::Vec3List,
            Self::Vec4(_) => ValueType::Vec4List,
        }
    }

    /// Convert to Value (clones the Arc, O(1))
    pub fn to_value(&self) -> Value {
        match self {
            Self::Float(v) => Value::FloatList(ArcVec(v.clone())),
            Self::Int(v) => Value::IntList(ArcVec(v.clone())),
            Self::Vec2(v) => Value::Vec2List(ArcVec(v.clone())),
            Self::Vec3(v) => Value::Vec3List(ArcVec(v.clone())),
            Self::Vec4(v) => Value::Vec4List(ArcVec(v.clone())),
        }
    }
}

impl AttributeTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a column by name
    pub fn get(&self, name: &str) -> Option<&AttributeColumn> {
        self.columns.get(name)
    }

    /// Get Vec3 column as Arc (O(1), no copy)
    pub fn get_vec3_arc(&self, name: &str) -> Option<Arc<Vec<[f32; 3]>>> {
        match self.get(name)? {
            AttributeColumn::Vec3(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Get Vec2 column as Arc (O(1), no copy)
    pub fn get_vec2_arc(&self, name: &str) -> Option<Arc<Vec<[f32; 2]>>> {
        match self.get(name)? {
            AttributeColumn::Vec2(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Get Float column as Arc (O(1), no copy)
    pub fn get_float_arc(&self, name: &str) -> Option<Arc<Vec<f32>>> {
        match self.get(name)? {
            AttributeColumn::Float(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Set a column (unchecked - caller ensures length is valid)
    pub(crate) fn set_unchecked(&mut self, name: String, value: Value) {
        let column = match value {
            Value::FloatList(v) => AttributeColumn::Float(v.into_arc()),
            Value::IntList(v) => AttributeColumn::Int(v.into_arc()),
            Value::Vec2List(v) => AttributeColumn::Vec2(v.into_arc()),
            Value::Vec3List(v) => AttributeColumn::Vec3(v.into_arc()),
            Value::Vec4List(v) => AttributeColumn::Vec4(v.into_arc()),
            _ => return, // Ignore non-list types
        };
        self.columns.insert(Arc::from(name), column);
    }

    /// Remove a column
    pub fn remove(&mut self, name: &str) -> Option<AttributeColumn> {
        self.columns.remove(name)
    }

    /// Check if table contains an attribute
    pub fn contains(&self, name: &str) -> bool {
        self.columns.contains_key(name)
    }

    /// Iterate over all columns
    pub fn iter(&self) -> impl Iterator<Item = (&str, &AttributeColumn)> {
        self.columns.iter().map(|(k, v)| (k.as_ref(), v))
    }

    /// Get column names
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.columns.keys().map(|k| k.as_ref())
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}
```

---

## DetailAttrs

Single-value attributes for whole-geometry data:

```rust
/// Detail attributes store single values for the whole geometry.
/// Unlike AttributeTable, these are scalars not lists.
#[derive(Clone, Debug, Default)]
pub struct DetailAttrs {
    values: HashMap<Arc<str>, Value>,
}

impl DetailAttrs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.values.insert(Arc::from(name), value);
    }

    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.values.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.values.iter().map(|(k, v)| (k.as_ref(), v))
    }
}
```

---

## Value Enum Additions

List types now use `ArcVec` wrapper for O(1) cloning:

```rust
/// Arc-wrapped vector for O(1) cloning in Value enum
#[derive(Clone, Debug)]
pub struct ArcVec<T>(pub Arc<Vec<T>>);

impl<T> ArcVec<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self(Arc::new(data))
    }

    pub fn into_arc(self) -> Arc<Vec<T>> {
        self.0
    }
}

impl<T> std::ops::Deref for ArcVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T: PartialEq> PartialEq for ArcVec<T> {
    fn eq(&self, other: &Self) -> bool {
        // Fast path: same Arc
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

pub enum Value {
    // ... existing variants ...

    // Geometry bundle
    Geometry(Arc<Geometry>),

    // List types with Arc wrapping for O(1) clone
    FloatList(ArcVec<f32>),
    IntList(ArcVec<i32>),
    Vec2List(ArcVec<[f32; 2]>),
    Vec3List(ArcVec<[f32; 3]>),
    Vec4List(ArcVec<[f32; 4]>),
}

impl Value {
    /// Get length if this is a list type
    pub fn list_len(&self) -> Option<usize> {
        match self {
            Self::FloatList(v) => Some(v.len()),
            Self::IntList(v) => Some(v.len()),
            Self::Vec2List(v) => Some(v.len()),
            Self::Vec3List(v) => Some(v.len()),
            Self::Vec4List(v) => Some(v.len()),
            _ => None,
        }
    }
}
```

---

## GeometryError

```rust
#[derive(Clone, Debug, thiserror::Error)]
pub enum GeometryError {
    #[error("invalid attribute type for '{name}': expected {expected}, got {got}")]
    InvalidAttributeType {
        name: String,
        expected: &'static str,
        got: ValueType,
    },

    #[error("attribute '{name}' not found")]
    AttributeNotFound { name: String },

    #[error("length mismatch in {context}: expected {expected}, got {got}")]
    LengthMismatch {
        context: String,
        expected: usize,
        got: usize,
    },

    #[error("invalid primitive index: {index} (max {max})")]
    InvalidPrimitiveIndex { index: usize, max: usize },

    #[error("invalid point index: {index} (max {max})")]
    InvalidPointIndex { index: usize, max: usize },
}
```

---

## Memory Model

### Fine-Grained Copy-on-Write

Each component is independently Arc-wrapped:

```rust
// Modifying only positions - other components not cloned
let mut geo = some_shared_geometry.clone();  // Arc clone, O(1)
geo.points_mut()[0] = [1.0, 0.0, 0.0];        // Clones only points vec

// Modifying only normals - positions not cloned
geo.set_point_attr("N", new_normals)?;        // Clones only point_attrs
```

### Version-Based Dirty Tracking

```rust
// Cache previous version
let mut cached_version = 0u64;
let mut cached_gpu_mesh: Option<MeshHandle> = None;

// Each frame
fn update(&mut self, geometry: &Geometry, gpu: &mut Gpu) -> MeshHandle {
    if geometry.version() != self.cached_version {
        // Geometry changed - re-upload
        self.cached_gpu_mesh = Some(gpu.upload(geometry));
        self.cached_version = geometry.version();
    }
    self.cached_gpu_mesh.unwrap()
}
```

### When Cloning Happens

| Operation | What's Cloned |
|-----------|---------------|
| `geometry.clone()` | Nothing (Arc bumps only) |
| `geometry.points_mut()` | Only `points` vec if shared |
| `geometry.set_point_attr(...)` | Only `point_attrs` if shared |
| `geometry.merge(other)` | Components being extended |
| Bridge operator read | Nothing (returns Arc) |

---

## Thread Safety

All types are `Send + Sync`:

```rust
// Safe to send geometry to background thread for processing
let geo = Arc::new(geometry);
std::thread::spawn(move || {
    let bounds = geo.bounds();
    // ...
});

// Safe to share geometry across threads
let geo = Arc::new(geometry);
let geo2 = geo.clone();
std::thread::spawn(move || { /* use geo */ });
std::thread::spawn(move || { /* use geo2 */ });
```
