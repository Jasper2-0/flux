# Unified Geometry Execution

How Flux abstracts CPU/GPU execution from the user's mental model.

## Design Principle

Users operate on **geometry**, not on CPU arrays or GPU buffers. The execution location is an optimization detail managed by the system.

```
User's Mental Model              System's Execution
─────────────────────            ──────────────────────────────

    ┌─────────┐                  ┌─────────┐
    │ Sphere  │                  │Sphere   │ (CPU)
    └────┬────┘                  └────┬────┘
         │                            │
    ┌────▼────┐                  ┌────▼────┐
    │Displace │                  │ Upload  │ (auto-inserted)
    └────┬────┘                  └────┬────┘
         │                            │
    ┌────▼────┐                  ┌────▼────┐
    │ Render  │                  │Displace │ (GPU compute)
    └─────────┘                  └────┬────┘
                                      │
                                 ┌────▼────┐
                                 │ Render  │
                                 └─────────┘
```

The logical graph (left) is what users build. The physical graph (right) is what the system executes. Users never manually insert Upload/Download nodes.

---

## Unified Geometry Type

### User-Facing Handle

```rust
/// Opaque handle to geometry data.
///
/// Users interact with this type exclusively. They cannot access
/// the underlying storage directly—all operations go through
/// operators that the system dispatches appropriately.
#[derive(Clone)]
pub struct Geometry {
    id: GeometryId,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryId(u64);

impl Geometry {
    /// Metadata queries (always available, cached in registry)
    pub fn point_count(&self, ctx: &EvalContext) -> usize;
    pub fn primitive_count(&self, ctx: &EvalContext) -> usize;
    pub fn bounds(&self, ctx: &EvalContext) -> BoundingBox;
    pub fn has_attribute(&self, name: &str, ctx: &EvalContext) -> bool;
}
```

### Why Opaque Handles?

1. **Location independence** — User code doesn't know or care where data lives
2. **Lazy materialization** — Data transfers happen only when needed
3. **Caching** — System can keep multiple representations without user managing them
4. **Future-proofing** — Can add new storage backends (shared memory, network) without API changes

---

## Internal Storage

### Geometry Registry

The registry tracks all geometry instances and their physical storage:

```rust
pub struct GeometryRegistry {
    /// Monotonic ID generator
    next_id: AtomicU64,

    /// Metadata (point count, bounds, attribute names)
    /// Always available without materialization
    metadata: RwLock<HashMap<GeometryId, GeometryMetadata>>,

    /// Physical storage for each geometry
    storage: RwLock<HashMap<GeometryId, GeometryStorage>>,
}
```

### Geometry Storage

Each geometry can exist in CPU memory, GPU memory, or both:

```rust
pub struct GeometryStorage {
    /// CPU representation (if materialized)
    cpu: Option<CpuGeometry>,

    /// GPU representation (if uploaded)
    gpu: Option<GpuGeometry>,

    /// Which representation is authoritative
    canonical: CanonicalLocation,

    /// Version counters for cache invalidation
    cpu_version: u64,
    gpu_version: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CanonicalLocation {
    /// CPU data is authoritative (GPU may be stale or absent)
    Cpu,
    /// GPU data is authoritative (CPU may be stale or absent)
    Gpu,
    /// Both are in sync
    Synced,
}
```

### CPU Geometry Data

Full geometry data in CPU memory (as defined in `types.md`):

```rust
pub struct CpuGeometry {
    pub(crate) version: u64,
    pub(crate) points: Arc<Vec<[f32; 3]>>,
    pub(crate) vertices: Arc<Vec<u32>>,
    pub(crate) primitive_starts: Arc<Vec<u32>>,
    pub(crate) primitive_types: Arc<Vec<PrimitiveType>>,
    pub(crate) point_attrs: Arc<AttributeTable>,
    pub(crate) vertex_attrs: Arc<AttributeTable>,
    pub(crate) prim_attrs: Arc<AttributeTable>,
    pub(crate) detail_attrs: Arc<DetailAttrs>,
}
```

### GPU Geometry Data

Geometry stored in GPU buffers:

```rust
pub struct GpuGeometry {
    /// Vertex positions (vec3<f32>)
    pub(crate) position_buffer: BufferHandle,

    /// Index buffer for primitives
    pub(crate) index_buffer: Option<BufferHandle>,

    /// Additional vertex attributes
    pub(crate) attribute_buffers: HashMap<String, GpuAttributeBuffer>,

    /// Metadata for dispatch
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,

    /// Version for dirty tracking
    pub(crate) version: u64,
}

pub struct GpuAttributeBuffer {
    pub buffer: BufferHandle,
    pub format: AttributeFormat,
    pub stride: u32,
}

pub enum AttributeFormat {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Uint,
}
```

---

## Data Materialization

### Ensure CPU Data

When an operation needs CPU access:

```rust
impl GeometryStorage {
    pub fn ensure_cpu(&mut self, ctx: &EvalContext) -> &CpuGeometry {
        match self.canonical {
            CanonicalLocation::Cpu | CanonicalLocation::Synced => {
                // Already have valid CPU data
                self.cpu.as_ref().unwrap()
            }
            CanonicalLocation::Gpu => {
                // Must download from GPU
                let gpu = self.gpu.as_ref().unwrap();
                let cpu = ctx.download_geometry(gpu);
                self.cpu = Some(cpu);
                self.cpu_version = self.gpu_version;
                self.canonical = CanonicalLocation::Synced;
                self.cpu.as_ref().unwrap()
            }
        }
    }
}
```

### Ensure GPU Data

When an operation needs GPU access:

```rust
impl GeometryStorage {
    pub fn ensure_gpu(&mut self, ctx: &EvalContext) -> &GpuGeometry {
        match self.canonical {
            CanonicalLocation::Gpu | CanonicalLocation::Synced => {
                // Already have valid GPU data
                self.gpu.as_ref().unwrap()
            }
            CanonicalLocation::Cpu => {
                // Must upload to GPU
                let cpu = self.cpu.as_ref().unwrap();
                let gpu = ctx.upload_geometry(cpu);
                self.gpu = Some(gpu);
                self.gpu_version = self.cpu_version;
                self.canonical = CanonicalLocation::Synced;
                self.gpu.as_ref().unwrap()
            }
        }
    }
}
```

### Invalidation After Modification

When an operation produces new geometry:

```rust
impl GeometryRegistry {
    /// Store result of CPU operation
    pub fn store_cpu(&self, data: CpuGeometry) -> Geometry {
        let id = GeometryId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let version = data.version;

        self.storage.write().insert(id, GeometryStorage {
            cpu: Some(data),
            gpu: None,
            canonical: CanonicalLocation::Cpu,
            cpu_version: version,
            gpu_version: 0,
        });

        Geometry { id }
    }

    /// Store result of GPU compute operation
    pub fn store_gpu(&self, data: GpuGeometry) -> Geometry {
        let id = GeometryId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let version = data.version;

        self.storage.write().insert(id, GeometryStorage {
            cpu: None,
            gpu: Some(data),
            canonical: CanonicalLocation::Gpu,
            cpu_version: 0,
            gpu_version: version,
        });

        Geometry { id }
    }
}
```

---

## Operator Polymorphism

### Operator Trait

Operators declare what implementations they support:

```rust
pub trait GeometryOperator: Send + Sync {
    /// Human-readable name for debugging
    fn name(&self) -> &'static str;

    /// What implementations are available
    fn implementations(&self) -> ImplementationSet;

    /// Execute on CPU (if supported)
    fn execute_cpu(
        &self,
        inputs: &[&CpuGeometry],
        params: &dyn OperatorParams,
        ctx: &EvalContext,
    ) -> Option<CpuGeometry> {
        None  // Default: not supported
    }

    /// Execute on GPU via compute shader (if supported)
    fn execute_gpu_compute(
        &self,
        inputs: &[&GpuGeometry],
        params: &dyn OperatorParams,
        ctx: &mut GpuContext,
    ) -> Option<GpuGeometry> {
        None  // Default: not supported
    }
}

bitflags! {
    pub struct ImplementationSet: u8 {
        const CPU = 0b001;
        const GPU_COMPUTE = 0b010;
        const GPU_VERTEX = 0b100;  // Can run in vertex shader at render time
    }
}
```

### Example: Displace Operator

```rust
pub struct DisplaceOp;

impl GeometryOperator for DisplaceOp {
    fn name(&self) -> &'static str { "Displace" }

    fn implementations(&self) -> ImplementationSet {
        ImplementationSet::CPU | ImplementationSet::GPU_COMPUTE
    }

    fn execute_cpu(
        &self,
        inputs: &[&CpuGeometry],
        params: &dyn OperatorParams,
        ctx: &EvalContext,
    ) -> Option<CpuGeometry> {
        let geo = inputs[0];
        let frequency = params.get_f32("frequency")?;
        let amplitude = params.get_f32("amplitude")?;
        let offset = params.get_vec3("offset")?;

        // Clone with COW semantics
        let mut result = geo.clone();
        let points = Arc::make_mut(&mut result.points);

        for p in points.iter_mut() {
            let noise = simplex_noise_3d([
                p[0] * frequency + offset[0],
                p[1] * frequency + offset[1],
                p[2] * frequency + offset[2],
            ]);
            // Displace along normal (simplified: radial)
            let len = (p[0]*p[0] + p[1]*p[1] + p[2]*p[2]).sqrt();
            if len > 0.0 {
                let scale = 1.0 + noise * amplitude / len;
                p[0] *= scale;
                p[1] *= scale;
                p[2] *= scale;
            }
        }

        result.version += 1;
        Some(result)
    }

    fn execute_gpu_compute(
        &self,
        inputs: &[&GpuGeometry],
        params: &dyn OperatorParams,
        ctx: &mut GpuContext,
    ) -> Option<GpuGeometry> {
        let geo = inputs[0];
        let frequency = params.get_f32("frequency")?;
        let amplitude = params.get_f32("amplitude")?;
        let offset = params.get_vec3("offset")?;

        // Create output buffer
        let output_buffer = ctx.create_buffer(
            geo.vertex_count as u64 * 12,  // vec3<f32>
            BufferUsages::STORAGE | BufferUsages::VERTEX,
        );

        // Update uniforms
        ctx.write_uniform("displace_params", &DisplaceParams {
            frequency,
            amplitude,
            offset,
        });

        // Dispatch compute shader
        ctx.dispatch_compute(
            "displace",
            &[&geo.position_buffer, &output_buffer],
            (geo.vertex_count + 255) / 256,
        );

        Some(GpuGeometry {
            position_buffer: output_buffer,
            index_buffer: geo.index_buffer.clone(),
            attribute_buffers: geo.attribute_buffers.clone(),
            vertex_count: geo.vertex_count,
            index_count: geo.index_count,
            version: geo.version + 1,
        })
    }
}
```

### Example: Box Generator (CPU-Only)

Some operations only make sense on CPU:

```rust
pub struct BoxOp;

impl GeometryOperator for BoxOp {
    fn name(&self) -> &'static str { "Box" }

    fn implementations(&self) -> ImplementationSet {
        ImplementationSet::CPU  // Topology creation is CPU-only
    }

    fn execute_cpu(
        &self,
        _inputs: &[&CpuGeometry],
        params: &dyn OperatorParams,
        _ctx: &EvalContext,
    ) -> Option<CpuGeometry> {
        let size = params.get_vec3("size")?;
        let center = params.get_vec3("center").unwrap_or([0.0; 3]);
        let divisions = params.get_ivec3("divisions").unwrap_or([1, 1, 1]);

        Some(generate_box(size, center, divisions))
    }
}
```

---

## Execution Scheduler

### Scheduling Decision

The scheduler determines where each operation executes:

```rust
pub struct Scheduler {
    /// Threshold below which GPU overhead isn't worth it
    gpu_min_vertices: u32,

    /// Cache of downstream requirements per node
    downstream_cache: HashMap<NodeId, RequirementSet>,
}

impl Scheduler {
    pub fn choose_location(
        &self,
        op: &dyn GeometryOperator,
        input_geo: &Geometry,
        registry: &GeometryRegistry,
        downstream: &[NodeId],
    ) -> ExecutionLocation {
        let impls = op.implementations();
        let storage = registry.get_storage(input_geo.id);
        let vertex_count = registry.metadata(input_geo.id).vertex_count;

        // Rule 1: If only one implementation exists, use it
        if impls == ImplementationSet::CPU {
            return ExecutionLocation::Cpu;
        }
        if impls == ImplementationSet::GPU_COMPUTE {
            return ExecutionLocation::Gpu;
        }

        // Rule 2: Small geometry → CPU (transfer overhead dominates)
        if vertex_count < self.gpu_min_vertices {
            return ExecutionLocation::Cpu;
        }

        // Rule 3: Check downstream requirements
        let downstream_needs_gpu = downstream.iter().any(|id| {
            self.downstream_cache.get(id)
                .map(|r| r.contains(Requirement::GpuAccess))
                .unwrap_or(false)
        });

        let downstream_needs_cpu = downstream.iter().any(|id| {
            self.downstream_cache.get(id)
                .map(|r| r.contains(Requirement::CpuAccess))
                .unwrap_or(false)
        });

        // Rule 4: If downstream only needs GPU (render), prefer GPU
        if downstream_needs_gpu && !downstream_needs_cpu {
            return ExecutionLocation::Gpu;
        }

        // Rule 5: Stay where data currently is (avoid transfer)
        match storage.canonical {
            CanonicalLocation::Gpu => ExecutionLocation::Gpu,
            CanonicalLocation::Cpu => ExecutionLocation::Cpu,
            CanonicalLocation::Synced => {
                // Prefer GPU for large geometry
                if vertex_count > 10_000 {
                    ExecutionLocation::Gpu
                } else {
                    ExecutionLocation::Cpu
                }
            }
        }
    }
}

pub enum ExecutionLocation {
    Cpu,
    Gpu,
}
```

### Downstream Analysis

Before execution, analyze what each node's outputs feed into:

```rust
impl Scheduler {
    pub fn analyze_downstream(&mut self, graph: &Graph) {
        self.downstream_cache.clear();

        // Traverse graph in reverse topological order
        for node_id in graph.reverse_topo_order() {
            let node = graph.node(node_id);
            let mut requirements = RequirementSet::empty();

            // Check what this node needs
            if node.op.is_render() {
                requirements.insert(Requirement::GpuAccess);
            }
            if node.op.needs_cpu_readback() {
                requirements.insert(Requirement::CpuAccess);
            }

            // Propagate from downstream nodes
            for downstream_id in graph.outputs_of(node_id) {
                if let Some(downstream_reqs) = self.downstream_cache.get(&downstream_id) {
                    requirements.extend(downstream_reqs);
                }
            }

            self.downstream_cache.insert(node_id, requirements);
        }
    }
}
```

---

## Graph Compilation

### Logical to Physical

The graph compiler transforms the user's logical graph into an executable physical graph:

```rust
pub struct GraphCompiler {
    scheduler: Scheduler,
}

impl GraphCompiler {
    pub fn compile(&self, logical: &Graph, registry: &GeometryRegistry) -> PhysicalGraph {
        // 1. Analyze downstream requirements
        self.scheduler.analyze_downstream(logical);

        // 2. Assign execution locations
        let locations = self.assign_locations(logical, registry);

        // 3. Insert transfer nodes where needed
        let physical = self.insert_transfers(logical, &locations);

        // 4. Fuse compatible adjacent operations (optimization)
        let optimized = self.fuse_operations(physical);

        optimized
    }

    fn assign_locations(
        &self,
        graph: &Graph,
        registry: &GeometryRegistry,
    ) -> HashMap<NodeId, ExecutionLocation> {
        let mut locations = HashMap::new();

        for node_id in graph.topo_order() {
            let node = graph.node(node_id);

            // Get input geometry (if any)
            let input_geo = node.geometry_input()
                .and_then(|input| graph.resolve_geometry(input));

            let location = match input_geo {
                Some(geo) => {
                    let downstream: Vec<_> = graph.outputs_of(node_id).collect();
                    self.scheduler.choose_location(
                        node.op.as_ref(),
                        &geo,
                        registry,
                        &downstream,
                    )
                }
                None => ExecutionLocation::Cpu,  // Generators start on CPU
            };

            locations.insert(node_id, location);
        }

        locations
    }

    fn insert_transfers(
        &self,
        graph: &Graph,
        locations: &HashMap<NodeId, ExecutionLocation>,
    ) -> PhysicalGraph {
        let mut physical = PhysicalGraph::new();

        for node_id in graph.topo_order() {
            let node = graph.node(node_id);
            let location = locations[&node_id];

            // Check each input
            for (input_idx, input) in node.inputs.iter().enumerate() {
                if let Some(source_id) = input.source_node {
                    let source_location = locations[&source_id];

                    // Insert transfer if locations differ
                    if source_location != location {
                        match (source_location, location) {
                            (ExecutionLocation::Cpu, ExecutionLocation::Gpu) => {
                                physical.insert_upload(source_id, node_id, input_idx);
                            }
                            (ExecutionLocation::Gpu, ExecutionLocation::Cpu) => {
                                physical.insert_download(source_id, node_id, input_idx);
                            }
                            _ => {}
                        }
                    }
                }
            }

            physical.add_node(node_id, node.clone(), location);
        }

        physical
    }
}
```

### Physical Graph Execution

```rust
pub struct PhysicalGraph {
    nodes: Vec<PhysicalNode>,
    edges: Vec<PhysicalEdge>,
}

pub struct PhysicalNode {
    id: NodeId,
    op: Box<dyn GeometryOperator>,
    location: ExecutionLocation,
    kind: PhysicalNodeKind,
}

pub enum PhysicalNodeKind {
    /// Normal operator execution
    Operator,
    /// Auto-inserted CPU→GPU transfer
    Upload,
    /// Auto-inserted GPU→CPU transfer
    Download,
}

impl PhysicalGraph {
    pub fn execute(&self, ctx: &mut EvalContext) -> HashMap<NodeId, Geometry> {
        let mut results: HashMap<NodeId, Geometry> = HashMap::new();

        for node in &self.nodes {
            let result = match node.kind {
                PhysicalNodeKind::Upload => {
                    let input = self.get_input(node.id, 0, &results);
                    ctx.registry.ensure_gpu(input.id);
                    input  // Same geometry, now has GPU data
                }
                PhysicalNodeKind::Download => {
                    let input = self.get_input(node.id, 0, &results);
                    ctx.registry.ensure_cpu(input.id);
                    input  // Same geometry, now has CPU data
                }
                PhysicalNodeKind::Operator => {
                    let inputs = self.gather_inputs(node.id, &results);
                    self.execute_operator(node, &inputs, ctx)
                }
            };

            results.insert(node.id, result);
        }

        results
    }

    fn execute_operator(
        &self,
        node: &PhysicalNode,
        inputs: &[Geometry],
        ctx: &mut EvalContext,
    ) -> Geometry {
        match node.location {
            ExecutionLocation::Cpu => {
                let cpu_inputs: Vec<_> = inputs.iter()
                    .map(|g| ctx.registry.ensure_cpu(g.id))
                    .collect();
                let cpu_refs: Vec<_> = cpu_inputs.iter().collect();

                let result = node.op.execute_cpu(&cpu_refs, &node.params, ctx)
                    .expect("CPU execution failed");

                ctx.registry.store_cpu(result)
            }
            ExecutionLocation::Gpu => {
                let gpu_inputs: Vec<_> = inputs.iter()
                    .map(|g| ctx.registry.ensure_gpu(g.id))
                    .collect();
                let gpu_refs: Vec<_> = gpu_inputs.iter().collect();

                let result = node.op.execute_gpu_compute(&gpu_refs, &node.params, &mut ctx.gpu)
                    .expect("GPU execution failed");

                ctx.registry.store_gpu(result)
            }
        }
    }
}
```

---

## Async GPU Readback

### The Problem

GPU→CPU transfer (download) requires synchronization that stalls the pipeline:

```
Frame N:   [Compute]──[Download]──[Wait...]──[CPU Op]──[Render]
                                     ↑
                              Pipeline stall
```

### Solution: Staged Readback

For operations that need CPU access to GPU-computed geometry, use 1-frame latency:

```rust
pub struct StagedReadback {
    /// Pending readback requests
    pending: VecDeque<ReadbackRequest>,

    /// Completed readbacks available for use
    completed: HashMap<GeometryId, CpuGeometry>,
}

struct ReadbackRequest {
    geometry_id: GeometryId,
    staging_buffer: BufferHandle,
    frame_requested: u64,
}

impl StagedReadback {
    pub fn request(&mut self, geo: &GpuGeometry, ctx: &GpuContext) -> GeometryId {
        // Copy to staging buffer (GPU→GPU, fast)
        let staging = ctx.create_staging_buffer(geo.byte_size());
        ctx.copy_buffer(&geo.position_buffer, &staging);

        self.pending.push_back(ReadbackRequest {
            geometry_id: geo.id,
            staging_buffer: staging,
            frame_requested: ctx.frame_number(),
        });

        geo.id
    }

    pub fn poll(&mut self, current_frame: u64, ctx: &GpuContext) {
        // Check if any pending readbacks are ready (at least 1 frame old)
        while let Some(req) = self.pending.front() {
            if current_frame > req.frame_requested {
                let req = self.pending.pop_front().unwrap();

                // Map staging buffer and read data
                let data = ctx.map_buffer_sync(&req.staging_buffer);
                let cpu_geo = CpuGeometry::from_bytes(&data);

                self.completed.insert(req.geometry_id, cpu_geo);
            } else {
                break;
            }
        }
    }

    pub fn get(&self, id: GeometryId) -> Option<&CpuGeometry> {
        self.completed.get(&id)
    }
}
```

### Usage Pattern

```
Frame N:   [GPU Compute]──[Request Readback]──[Render]
Frame N+1: [Poll Readback]──[CPU Op uses N's data]──[Render]
```

The CPU operation sees geometry from the previous frame. For most use cases (collision detection, UI display of values), this 1-frame latency is acceptable.

---

## User-Facing API

### Building Graphs

Users work with the logical graph API:

```rust
// User code - completely location-agnostic
fn build_graph(graph: &mut Graph) {
    // Generator (will run on CPU)
    let sphere = graph.add(Sphere {
        radius: 1.0,
        rows: 64,
        columns: 32,
    });

    // Modifier (system chooses CPU or GPU)
    let displaced = graph.add(Displace {
        frequency: 4.0,
        amplitude: 0.2,
    });

    // Another modifier
    let transformed = graph.add(Transform {
        matrix: Matrix4::rotation_y(time),
    });

    // Render (needs GPU)
    let render = graph.add(Render {
        material: material_handle,
    });

    // Connect the graph
    graph.connect(sphere.output(0), displaced.input(0));
    graph.connect(displaced.output(0), transformed.input(0));
    graph.connect(transformed.output(0), render.input(0));

    // Animate
    graph.connect(time_node.output(0), displaced.input("offset"));
}
```

### Forcing Execution Location

Escape hatch for debugging or specific requirements:

```rust
// Force CPU execution (useful for debugging)
let displaced = graph.add(Displace {
    frequency: 4.0,
    amplitude: 0.2,
}.force_cpu());

// Force GPU execution (when you know better than the scheduler)
let transformed = graph.add(Transform {
    matrix: Matrix4::identity(),
}.force_gpu());
```

### Inspecting Execution Plan

For debugging and profiling:

```rust
let physical = compiler.compile(&graph, &registry);

for node in physical.nodes() {
    println!(
        "{}: {} on {:?}",
        node.id,
        node.op.name(),
        node.location,
    );
}

// Output:
// 0: Sphere on Cpu
// 1: Upload on Gpu  (auto-inserted)
// 2: Displace on Gpu
// 3: Transform on Gpu
// 4: Render on Gpu
```

---

## Integration with Dirty Tracking

### Version Propagation

The existing version-based dirty tracking extends to GPU data:

```rust
impl GeometryStorage {
    pub fn is_cpu_dirty(&self) -> bool {
        self.canonical == CanonicalLocation::Gpu &&
        self.cpu_version < self.gpu_version
    }

    pub fn is_gpu_dirty(&self) -> bool {
        self.canonical == CanonicalLocation::Cpu &&
        self.gpu_version < self.cpu_version
    }
}
```

### Frame-Level Caching

The physical graph caches results within a frame:

```rust
impl PhysicalGraph {
    pub fn execute_cached(
        &self,
        ctx: &mut EvalContext,
        input_versions: &HashMap<NodeId, u64>,
    ) -> HashMap<NodeId, Geometry> {
        let mut results = HashMap::new();

        for node in &self.nodes {
            // Check if inputs changed
            let inputs_changed = self.inputs_of(node.id)
                .any(|input_id| {
                    let current = input_versions.get(&input_id).copied().unwrap_or(0);
                    let cached = self.cached_versions.get(&input_id).copied().unwrap_or(0);
                    current != cached
                });

            if inputs_changed || !self.cache.contains_key(&node.id) {
                // Re-execute
                let result = self.execute_node(node, &results, ctx);
                self.cache.insert(node.id, result.clone());
                results.insert(node.id, result);
            } else {
                // Use cached result
                results.insert(node.id, self.cache[&node.id].clone());
            }
        }

        results
    }
}
```

---

## Summary

| Concept | Description |
|---------|-------------|
| **Unified Geometry** | Opaque handle, user doesn't see CPU/GPU distinction |
| **GeometryStorage** | Internal dual-representation with canonical tracking |
| **Operator Polymorphism** | Operators declare CPU/GPU implementations |
| **Scheduler** | Chooses execution location based on heuristics |
| **Graph Compiler** | Transforms logical → physical graph, inserts transfers |
| **Staged Readback** | 1-frame latency for GPU→CPU without stalls |

The user builds logical graphs. The system compiles to optimal physical execution.
