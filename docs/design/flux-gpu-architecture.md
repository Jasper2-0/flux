# Flux GPU Architecture

## Overview

This document describes the architecture for adding GPU support to Flux, enabling efficient texture generation and processing through the node graph system.

## Design Goals

1. **Transparent to users** — Node graph builders don't think about CPU vs GPU
2. **Efficient caching** — Dirty flag system drives GPU resource invalidation
3. **Clean ownership** — Applications own GPU resources, Flux uses them
4. **Flexible foundation** — Can refactor to support multiple backends later

---

## Crate Structure

```
solar-gpu (depends on wgpu)
│   Independent GPU infrastructure crate
│   No dependency on Flux
│
├── flux-gpu (depends on solar-gpu only)
│   │   GPU operator implementations
│   │   No direct wgpu dependency
│   │
│   └── flux-core, flux-graph, flux-operators (existing)
│
└── editor / player (application layer)
        Owns wgpu device, queue, surface
        Owns GpuResourcePool instance
        Owns winit event loop
```

### Why This Structure

- **solar-gpu independent**: Can be used for non-Flux applications
- **flux-gpu depends on solar-gpu, not wgpu**: Abstraction boundary exists for future backend swaps
- **Application owns resources**: For 64k intros or minimal builds, can swap out solar-gpu entirely

---

## solar-gpu Components

### GpuResourcePool

Manages GPU resource allocation and caching with multi-frame lifetimes.

```rust
pub struct GpuResourcePool {
    device: Arc<wgpu::Device>,
    textures: SlotMap<TextureKey, ManagedTexture>,
    buffers: SlotMap<BufferKey, ManagedBuffer>,
    pending_releases: Vec<PendingRelease>,
}

struct ManagedTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    size: (u32, u32),
    cpu_version: u64,
    gpu_version: u64,
}
```

**Responsibilities:**
- Allocate textures and buffers
- Track storage location (CPU-only, GPU-only, both)
- Track version per location (tixl-style reference/target)
- Batch transfers at frame boundaries
- Deferred reclamation after GPU completion

### GpuFrame

Per-frame state for command batching.

```rust
pub struct GpuFrame {
    encoder: wgpu::CommandEncoder,
    pending_uploads: Vec<PendingUpload>,
    pending_readbacks: Vec<PendingReadback>,
}

impl GpuFrame {
    pub fn new(device: &wgpu::Device) -> Self;
    pub fn finish(self) -> wgpu::CommandBuffer;
}
```

**Lifecycle:**
1. Created at start of update()
2. Operators record commands to encoder
3. Submitted in draw()
4. Discarded after submission

### GpuHandle<T>

Opaque handle to GPU resources.

```rust
pub struct GpuHandle<T> {
    key: SlotMapKey,
    generation: u32,
    _marker: PhantomData<T>,
}

impl<T> GpuHandle<T> {
    pub fn is_valid(&self, pool: &GpuResourcePool) -> bool;
}
```

### Staleness Tracking

Tixl-style reference/target pattern with frame deduplication:

```rust
struct CachedResource {
    handle: GpuHandle<Texture>,
    reference: u64,  // version when this cache was created
}

struct Operator {
    target: u64,  // incremented when inputs change
    cached: Option<CachedResource>,
}

impl Operator {
    fn is_stale(&self) -> bool {
        match &self.cached {
            Some(c) => c.reference < self.target,
            None => true,
        }
    }
}
```

---

## flux-gpu Components

### Transparent CPU/GPU Values

Users see unified types; storage location is internal:

```rust
pub enum TextureStorage {
    Cpu { data: Vec<u8>, format: TextureFormat },
    Gpu { handle: GpuHandle<Texture> },
    Both {
        cpu: Vec<u8>,
        gpu: GpuHandle<Texture>,
        authoritative: Location,
    },
}

pub struct TextureValue {
    storage: TextureStorage,
    width: u32,
    height: u32,
    format: TextureFormat,
}
```

### Automatic Transfers

When a GPU operator receives CPU data (or vice versa), transfer is automatic:

```rust
impl TextureValue {
    pub fn ensure_gpu(
        &mut self,
        pool: &GpuResourcePool,
        frame: &mut GpuFrame,
    ) -> GpuHandle<Texture> {
        match &self.storage {
            TextureStorage::Gpu { handle } => handle.clone(),
            TextureStorage::Cpu { data, .. } => {
                let handle = pool.allocate_texture(...);
                frame.upload_texture(&handle, data);
                self.storage = TextureStorage::Both { ... };
                handle
            }
            TextureStorage::Both { gpu, .. } => gpu.clone(),
        }
    }

    pub fn ensure_cpu(
        &mut self,
        pool: &GpuResourcePool,
        frame: &mut GpuFrame,
    ) -> &[u8] {
        // Similar logic for readback
    }
}
```

### GPU Operator Pattern

```rust
pub struct BlurOp {
    input: InputPort<TextureValue>,
    radius: InputPort<f32>,
    output: OutputPort<TextureValue>,

    // Caching
    cached_output: Option<CachedResource>,
    pipeline: Option<wgpu::ComputePipeline>,
}

impl Operator for BlurOp {
    fn eval(&mut self, ctx: &EvalContext, gpu: &mut GpuContext) -> Result<()> {
        // Skip if clean
        if !self.is_dirty() && self.cached_output.is_some() {
            self.output.set(self.cached_output.as_ref().unwrap().as_value());
            return Ok(());
        }

        // Get input (auto-uploads if CPU)
        let input = self.input.get().ensure_gpu(&gpu.pool, &gpu.frame);
        let radius = self.radius.get();

        // Allocate output
        let output = gpu.pool.allocate_texture(...);

        // Record compute pass
        gpu.frame.compute_pass(|pass| {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, ...);
            pass.dispatch_workgroups(...);
        });

        // Cache and output
        self.cached_output = Some(CachedResource::new(output.clone()));
        self.output.set(TextureValue::from_gpu(output));

        Ok(())
    }
}
```

---

## Texture Format Strategy

### Internal Processing

```rust
const INTERNAL_COLOR_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
const INTERNAL_MASK_FORMAT: TextureFormat = TextureFormat::R16Float;
const INTERNAL_VECTOR_FORMAT: TextureFormat = TextureFormat::Rg16Float;
```

**Rationale:**
- Float avoids rounding errors and banding through multiple operations
- 16-bit float is filterable on all backends (32-bit requires feature check)
- Half the memory of 32-bit with sufficient precision for texture work
- HDR values (>1.0) preserved until output

### Output Conversion

```rust
const OUTPUT_COLOR_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
const OUTPUT_DATA_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const OUTPUT_MASK_FORMAT: TextureFormat = TextureFormat::R8Unorm;
```

Output nodes handle:
- Linear → sRGB conversion
- Float → integer quantization
- Optional dithering to reduce banding

---

## Execution Model

### Frame Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│  update()                                                   │
│                                                             │
│  1. Create GpuFrame                                         │
│  2. Evaluate flux graph                                     │
│     - Clean operators return cached handles                 │
│     - Dirty operators record commands to frame              │
│     - Auto-transfers queued as needed                       │
│  3. GpuFrame holds recorded commands                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  draw()                                                     │
│                                                             │
│  1. Submit GpuFrame command buffer                          │
│  2. Render preview (if applicable)                          │
│  3. Present surface                                         │
│  4. Process deferred releases                               │
└─────────────────────────────────────────────────────────────┘
```

### Dirty Flag Integration

```
User changes parameter
         │
         ▼
Operator marked dirty
         │
         ▼
Operator.target version incremented
         │
         ▼
Next eval: is_stale() returns true
         │
         ▼
Operator re-records GPU commands
         │
         ▼
New output cached, reference = target
         │
         ▼
Downstream operators also dirty (inputs changed)
```

---

## Application Integration

### Editor / Player Setup

```rust
fn main() {
    // Application owns wgpu
    let instance = wgpu::Instance::new(...);
    let adapter = instance.request_adapter(...).await;
    let (device, queue) = adapter.request_device(...).await;

    // Application owns resource pool
    let pool = GpuResourcePool::new(device.clone());

    // Application owns flux graph
    let graph = FluxGraph::load("project.flux");

    // Event loop
    event_loop.run(|event, _| {
        match event {
            Event::MainEventsCleared => {
                // update()
                let mut frame = GpuFrame::new(&device);
                graph.evaluate(&eval_ctx, &pool, &mut frame);

                // draw()
                let commands = frame.finish();
                queue.submit([commands]);

                pool.process_deferred_releases();
            }
            _ => {}
        }
    });
}
```

---

## Future Considerations

### Backend Abstraction (Later)

Current: `solar-gpu` depends on wgpu directly.

Future option:
```
solar-gpu (traits only)
solar-gpu-wgpu (implements for wgpu)
solar-gpu-vulkano (alternative)
```

The crate boundary is in place; extraction can happen when needed.

### Graph-Level Optimization (Later)

- Analyze graph to minimize CPU/GPU transfers
- Batch similar operations
- Reorder for better GPU utilization

### Resolution Independence (Later)

- Logical vs actual texture size
- Automatic re-evaluation on resolution change
- Progressive refinement for preview
