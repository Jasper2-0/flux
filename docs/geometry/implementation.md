# Implementation Roadmap

Phased implementation plan for the Flux geometry system.

## Phase Overview

| Phase | Focus | Deliverable |
|-------|-------|-------------|
| 1 | Core Types | `Geometry`, `AttributeTable`, Value additions |
| 2 | Generators | Box, Sphere, Grid, Line |
| 3 | Bridge Operators | GetPositions, GetNormals, GetAttribute |
| 4 | GPU Integration | UploadGeometry, render pipeline |
| 5 | Modifiers | Transform, Merge |
| 6 | Unified Execution | Scheduler, graph compiler, operator polymorphism |
| 7 | Compute Shaders | GPU-side geometry ops (Displace, Particles) |
| 8 | Future | Instancing, subdivision, curves |

---

## Phase 1: Core Types

**Goal:** Add geometry types to flux-core.

### Files to Modify

```
flux-core/src/value/
├── mod.rs          # Add Value::Geometry, Vec2List, Vec4List
├── geometry.rs     # NEW: Geometry struct, AttributeTable
```

### Tasks

1. **Create `geometry.rs`**
   - [ ] `Geometry` struct with points, vertices, primitive_starts, primitive_types
   - [ ] `AttributeTable` wrapper around HashMap<String, Value>
   - [ ] `PrimitiveType` enum (Polygon, Polyline, Point)
   - [ ] `AttributeLevel` enum (Point, Vertex, Primitive, Detail)
   - [ ] `GeometryError` enum
   - [ ] Core methods: `new()`, `point_count()`, `primitive_count()`, `bounds()`

2. **Update `mod.rs`**
   - [ ] Add `Value::Geometry(Arc<Geometry>)`
   - [ ] Add `Value::Vec2List(Vec<[f32; 2]>)`
   - [ ] Add `Value::Vec4List(Vec<[f32; 4]>)`
   - [ ] Add `ValueType::Geometry`, `ValueType::Vec2List`, `ValueType::Vec4List`
   - [ ] Update `TypeCategory::List` to include new list types
   - [ ] Add `TypeCategory::Geometry`
   - [ ] Add `as_geometry()`, `as_vec2_list()`, `as_vec4_list()` accessors
   - [ ] Add `From<Geometry>`, `From<Vec<[f32; 2]>>` implementations

### Verification

```rust
#[test]
fn test_geometry_creation() {
    let mut geo = Geometry::new();
    geo.add_triangle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]);
    assert_eq!(geo.point_count(), 3);
    assert_eq!(geo.primitive_count(), 1);
}

#[test]
fn test_value_geometry() {
    let geo = Geometry::new();
    let value = Value::Geometry(Arc::new(geo));
    assert_eq!(value.value_type(), ValueType::Geometry);
    assert!(value.as_geometry().is_some());
}
```

---

## Phase 2: Generators

**Goal:** Create geometry from parameters.

### Files to Create

```
flux-operators/src/geometry/
├── mod.rs
├── box_geometry.rs
├── sphere_geometry.rs
├── grid_geometry.rs
├── line_geometry.rs
```

### Tasks

1. **BoxGeometry**
   - [ ] Generate 8 points, 12 triangles (6 faces × 2 triangles)
   - [ ] Support subdivisions per axis
   - [ ] Generate face normals
   - [ ] Unit tests

2. **SphereGeometry**
   - [ ] Generate UV sphere with configurable rows/columns
   - [ ] Generate smooth normals (pointing outward)
   - [ ] Generate UV coordinates
   - [ ] Unit tests

3. **GridGeometry**
   - [ ] Generate flat XZ grid
   - [ ] Support rows/columns configuration
   - [ ] Generate Y-up normals
   - [ ] Generate UVs
   - [ ] Unit tests

4. **LineGeometry**
   - [ ] Generate polyline primitive
   - [ ] Support point count for interpolation
   - [ ] Unit tests

### Verification

```rust
#[test]
fn test_box_geometry() {
    let mut op = BoxGeometry::new();
    let result = op.evaluate(&[
        Value::Vec3([2.0, 2.0, 2.0]),
        Value::Vec3([0.0, 0.0, 0.0]),
        Value::Vec3([1.0, 1.0, 1.0]),
    ], &EvalContext::new());

    let geo = result[0].as_geometry().unwrap();
    assert_eq!(geo.point_count(), 8);
    assert_eq!(geo.primitive_count(), 12); // 6 faces × 2 triangles
}
```

---

## Phase 3: Bridge Operators

**Goal:** Extract and inject data from/to geometry.

### Files to Create

```
flux-operators/src/geometry/
├── get_positions.rs
├── get_normals.rs
├── get_attribute.rs
├── set_attribute.rs
├── point_count.rs
```

### Tasks

1. **GeometryGetPositions**
   - [ ] Extract `geometry.points` as `Vec3List`
   - [ ] Handle empty geometry

2. **GeometryGetNormals**
   - [ ] Extract `point_attrs["N"]` as `Vec3List`
   - [ ] Return empty list if no normals

3. **GeometryGetAttribute**
   - [ ] String input for attribute name
   - [ ] AttributeLevel input for which table
   - [ ] Return attribute column or error

4. **GeometrySetAttribute**
   - [ ] Validate data length matches element count
   - [ ] Clone geometry, set attribute, return new geometry
   - [ ] Error handling for type/length mismatches

5. **GeometryPointCount**
   - [ ] Return `geometry.point_count()` as Int

### Verification

```rust
#[test]
fn test_get_positions() {
    let geo = /* box geometry */;
    let mut op = GeometryGetPositions::new();
    let result = op.evaluate(&[Value::Geometry(Arc::new(geo))], &ctx);

    let positions = result[0].as_vec3_list().unwrap();
    assert_eq!(positions.len(), 8);
}

#[test]
fn test_roundtrip_attribute() {
    let geo = /* geometry with normals */;

    // Get normals
    let normals = GeometryGetNormals::evaluate(&geo);

    // Modify normals
    let modified: Vec<_> = normals.iter().map(|n| [-n[0], -n[1], -n[2]]).collect();

    // Set back
    let geo2 = GeometrySetAttribute::evaluate(&geo, "N", AttributeLevel::Point, modified);

    // Verify
    let normals2 = GeometryGetNormals::evaluate(&geo2);
    assert_eq!(normals2[0], [-normals[0][0], -normals[0][1], -normals[0][2]]);
}
```

---

## Phase 4: GPU Integration

**Goal:** Upload geometry for rendering.

### Files to Modify

```
flux-operators/src/geometry/
├── upload_geometry.rs    # NEW

# Integration with existing render system
flux-graph/src/...        # May need render context changes
```

### Tasks

1. **UploadGeometry Operator**
   - [ ] Create GPU vertex buffer from positions
   - [ ] Create GPU index buffer from primitives
   - [ ] Upload normals, UVs, colors as vertex attributes
   - [ ] Return MeshHandle

2. **Dirty Flag Integration**
   - [ ] Only re-upload when input geometry changed
   - [ ] Cache MeshHandle between frames
   - [ ] Invalidate cache when geometry dirty

3. **Vertex Layout**
   - [ ] Define standard vertex struct
   - [ ] Handle missing attributes (use defaults)
   - [ ] Pack into interleaved buffer

### Verification

```rust
#[test]
fn test_upload_geometry() {
    let geo = BoxGeometry::generate();
    let mut op = UploadGeometry::new();

    // First upload
    let handle1 = op.evaluate(&[Value::Geometry(Arc::new(geo.clone()))], &ctx);
    assert!(handle1[0].as_mesh_handle().is_some());

    // Second call with same geometry should return cached handle
    let handle2 = op.evaluate(&[Value::Geometry(Arc::new(geo))], &ctx);
    assert_eq!(handle1[0], handle2[0]); // Same handle, no re-upload
}
```

### Integration Test

```rust
// End-to-end: generate box, upload, verify renders
#[test]
fn test_box_render_pipeline() {
    let mut graph = Graph::new();

    let box_op = graph.add(BoxGeometry::new());
    let upload = graph.add(UploadGeometry::new());
    graph.connect(box_op, 0, upload, 0);

    let ctx = EvalContext::new();
    let handle = graph.evaluate(upload, 0, &ctx);

    assert!(handle.as_mesh_handle().is_some());
    // Render with handle and verify no errors
}
```

---

## Phase 5: Modifiers

**Goal:** Transform and combine geometry.

### Files to Create

```
flux-operators/src/geometry/
├── transform.rs
├── merge.rs
```

### Tasks

1. **GeometryTransform**
   - [ ] Transform positions with full matrix
   - [ ] Transform normals with inverse-transpose
   - [ ] Transform other Vec3 attributes (rotate+scale only)
   - [ ] Preserve other attributes unchanged

2. **GeometryMerge**
   - [ ] Concatenate points with index offset
   - [ ] Concatenate vertices with offset
   - [ ] Concatenate primitives
   - [ ] Merge attribute tables (concatenate columns, pad missing)

### Verification

```rust
#[test]
fn test_transform() {
    let geo = BoxGeometry::generate_unit();
    let matrix = Matrix4::translation([5.0, 0.0, 0.0]);

    let transformed = GeometryTransform::evaluate(&geo, &matrix);

    // All points should be offset by 5 in X
    for p in &transformed.points {
        assert!(p[0] >= 4.5); // Original was -0.5..0.5, now 4.5..5.5
    }
}

#[test]
fn test_merge() {
    let geo_a = BoxGeometry::generate();
    let geo_b = SphereGeometry::generate();

    let merged = GeometryMerge::evaluate(&geo_a, &geo_b);

    assert_eq!(merged.point_count(), geo_a.point_count() + geo_b.point_count());
    assert_eq!(merged.primitive_count(), geo_a.primitive_count() + geo_b.primitive_count());
}
```

---

## Phase 6: Unified Execution

**Goal:** Abstract CPU/GPU execution from user's mental model.

See [execution.md](execution.md) for full design.

### Files to Create

```
flux-graph/src/
├── scheduler.rs       # Execution location decisions
├── compiler.rs        # Logical → physical graph transformation
├── registry.rs        # GeometryRegistry, GeometryStorage

flux-core/src/value/
├── geometry.rs        # UPDATE: Add GeometryId, GpuGeometry
```

### Tasks

1. **GeometryRegistry**
   - [ ] `GeometryId` opaque handle type
   - [ ] `GeometryStorage` with CPU/GPU dual representation
   - [ ] `CanonicalLocation` enum (Cpu, Gpu, Synced)
   - [ ] `ensure_cpu()` / `ensure_gpu()` materialization
   - [ ] Version tracking for cache invalidation

2. **Operator Polymorphism**
   - [ ] `GeometryOperator` trait with `implementations()` method
   - [ ] `ImplementationSet` bitflags (CPU, GPU_COMPUTE, GPU_VERTEX)
   - [ ] `execute_cpu()` / `execute_gpu_compute()` methods
   - [ ] Update existing operators to implement trait

3. **Scheduler**
   - [ ] Heuristics: vertex count threshold, downstream analysis
   - [ ] `choose_location()` decision function
   - [ ] `analyze_downstream()` requirement propagation

4. **Graph Compiler**
   - [ ] `compile()` logical → physical transformation
   - [ ] Auto-insert Upload/Download nodes
   - [ ] Physical graph execution

5. **Staged Readback**
   - [ ] `StagedReadback` for GPU→CPU without stalls
   - [ ] 1-frame latency pattern

### Verification

```rust
#[test]
fn test_auto_upload() {
    let mut graph = Graph::new();
    let sphere = graph.add(Sphere::new(1.0, 32, 16));
    let render = graph.add(Render::new());
    graph.connect(sphere, render);

    let physical = compiler.compile(&graph, &registry);

    // Should auto-insert upload between CPU sphere and GPU render
    assert!(physical.has_upload_node());
}

#[test]
fn test_gpu_chain() {
    let mut graph = Graph::new();
    let sphere = graph.add(Sphere::new(1.0, 64, 32));
    let displace = graph.add(Displace::new(2.0, 0.3));
    let render = graph.add(Render::new());
    graph.connect(sphere, displace);
    graph.connect(displace, render);

    let physical = compiler.compile(&graph, &registry);

    // Sphere on CPU, upload once, displace on GPU, render
    assert_eq!(physical.location_of(sphere), ExecutionLocation::Cpu);
    assert_eq!(physical.location_of(displace), ExecutionLocation::Gpu);
}
```

---

## Phase 7: Compute Shaders

**Goal:** GPU-side geometry modification via wgpu compute.

### Files to Create

```
flux-operators/src/compute/
├── mod.rs
├── displace.rs        # Noise-based displacement
├── normal_recalc.rs   # Recompute normals after modification
├── particle.rs        # Particle system update
```

### Tasks

1. **Compute Infrastructure**
   - [ ] Compute shader loading and compilation
   - [ ] Storage buffer management for geometry
   - [ ] Uniform buffer for operator parameters
   - [ ] Workgroup dispatch helpers

2. **DisplaceCompute**
   - [ ] WGSL shader for noise displacement
   - [ ] Operator with CPU fallback and GPU compute impl
   - [ ] Output buffer management (double-buffering)

3. **NormalRecalcCompute**
   - [ ] Recompute vertex normals from positions
   - [ ] Handle indexed geometry

4. **ParticleUpdate**
   - [ ] Position/velocity integration
   - [ ] Age/lifetime management
   - [ ] Emit/destroy logic

### Verification

```rust
#[test]
fn test_displace_gpu() {
    let geo = SphereGeometry::generate(1.0, 64, 32);
    let registry = GeometryRegistry::new();
    let id = registry.store_cpu(geo);

    // Force GPU execution
    let displace = DisplaceOp.force_gpu();
    let result = displace.execute(&[id], &params, &mut ctx);

    // Result should be GPU-canonical
    let storage = registry.get_storage(result.id);
    assert_eq!(storage.canonical, CanonicalLocation::Gpu);
}

#[test]
fn test_compute_chain() {
    // Displace → NormalRecalc → Render (all on GPU, no transfers)
    let physical = compile_graph(&graph);

    let transfers = physical.nodes()
        .filter(|n| matches!(n.kind, PhysicalNodeKind::Upload | PhysicalNodeKind::Download))
        .count();

    assert_eq!(transfers, 1);  // Only initial upload
}
```

---

## Phase 8: Future Enhancements

### Instancing

Render thousands of copies efficiently:

```rust
pub struct InstanceGeometry {
    // Inputs: geometry, positions (Vec3List), scales (FloatList), rotations (Vec4List)
    // Output: InstancedMeshHandle (GPU instanced rendering)
}
```

### Subdivision

Catmull-Clark subdivision:

```rust
pub struct GeometrySubdivide {
    // Input: geometry, iterations
    // Output: smoothed geometry
}
```

### Curves

Bezier and NURBS curves:

```rust
pub struct CurveGeometry {
    control_points: Vec<[f32; 3]>,
    degree: u32,
    closed: bool,
}

pub struct CurveSample {
    // Input: curve, t (0..1)
    // Output: Vec3 (position on curve)
}
```

### Boolean Operations

CSG union/intersect/subtract:

```rust
pub struct GeometryBoolean {
    // Inputs: geometry_a, geometry_b, operation (Union/Intersect/Subtract)
    // Output: geometry
}
```

---

## Dependency Graph

```
Phase 1: Core Types
    │
    ├──▶ Phase 2: Generators (depends on Geometry type)
    │         │
    │         └──▶ Phase 4: GPU (depends on generators for testing)
    │                   │
    │                   └──▶ Phase 6: Unified Execution (depends on GPU integration)
    │                             │
    │                             └──▶ Phase 7: Compute Shaders (depends on execution model)
    │
    └──▶ Phase 3: Bridge Operators (depends on Geometry type)
              │
              └──▶ Phase 5: Modifiers (uses bridge operators internally)
```

**Recommended order:** 1 → 2 → 3 → 4 → 5 → 6 → 7

- Phases 1-5 establish the CPU-side geometry system
- Phase 6 adds the abstraction layer for transparent execution
- Phase 7 adds GPU compute implementations that plug into Phase 6

---

## Estimated Effort

| Phase | Files | Complexity | Notes |
|-------|-------|------------|-------|
| 1 | 2 | Medium | Core types, careful API design |
| 2 | 5 | Medium | Mesh generation algorithms |
| 3 | 5 | Low | Simple data extraction |
| 4 | 2 | High | GPU integration, caching |
| 5 | 2 | Medium | Transform math, merge logic |
| 6 | 4 | High | Scheduler, compiler, registry—architectural core |
| 7 | 4 | High | WGSL shaders, compute pipeline, buffer management |

**Total:** ~24 files across flux-core, flux-operators, and flux-graph.

**Implementation Note:** Phases 1-5 can be built and used without Phases 6-7. The unified execution model is an optimization layer—geometry works without it, just with manual CPU/GPU decisions.
