# Flux GPU Proof of Concept Plan

## Goal

Build a minimal proof of concept that demonstrates:
- GPU texture generation nodes working
- Graph defined in code (no editor)
- Final texture written to disk

## Success Criteria

```rust
fn main() {
    // Define graph in code
    let graph = TextureGraph::new()
        .add(PerlinNoise::new().frequency(4.0))
        .add(Levels::new().black(0.2).white(0.8))
        .add(GaussianBlur::new().radius(8.0))
        .add(Output::new("stone_height.png"));

    // Run and export
    graph.evaluate_and_export(1024, 1024);
}
```

Running this produces `stone_height.png` on disk.

---

## Implementation Phases

### Phase 1: Minimal solar-gpu

Create the bare minimum GPU infrastructure.

**Deliverables:**
- [ ] `solar-gpu` crate with Cargo.toml
- [ ] `GpuContext` — wraps device + queue (borrowed from app)
- [ ] `GpuFrame` — command encoder wrapper
- [ ] `TextureHandle` — simple handle to wgpu::Texture
- [ ] `allocate_texture(width, height, format)` — basic allocation
- [ ] `readback_texture(handle)` — GPU → CPU transfer

**Scope limits:**
- No pooling yet (allocate fresh each time)
- No caching yet (re-evaluate everything)
- No dirty tracking yet

```rust
// Minimal API for Phase 1
pub struct GpuContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub struct GpuFrame {
    encoder: wgpu::CommandEncoder,
}

impl GpuFrame {
    pub fn new(device: &wgpu::Device) -> Self;
    pub fn finish(self) -> wgpu::CommandBuffer;
}

pub struct TextureHandle {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}
```

---

### Phase 2: Basic GPU Operators

Implement a few texture generation operators.

**Deliverables:**
- [ ] `flux-gpu` crate with Cargo.toml
- [ ] `GpuOperator` trait (simplified for PoC)
- [ ] `PerlinNoise` operator — compute shader generates noise
- [ ] `SolidColor` operator — fills texture with color
- [ ] `Blend` operator — blends two textures

**Compute shader example (perlin noise):**
```wgsl
@group(0) @binding(0) var output: texture_storage_2d<rgba16float, write>;

struct Params {
    frequency: f32,
    octaves: u32,
    seed: u32,
}
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let uv = vec2<f32>(id.xy) / vec2<f32>(textureDimensions(output));
    let noise = perlin(uv * params.frequency, params.seed);
    textureStore(output, id.xy, vec4<f32>(noise, noise, noise, 1.0));
}
```

**Operator trait for PoC:**
```rust
pub trait GpuOperator {
    fn execute(
        &self,
        ctx: &GpuContext,
        frame: &mut GpuFrame,
        inputs: &[&TextureHandle],
        output: &TextureHandle,
    );
}
```

---

### Phase 3: Simple Graph + Execution

Wire operators into a simple graph structure.

**Deliverables:**
- [ ] `TextureGraph` — holds nodes and connections
- [ ] `Node` — wraps an operator with inputs/outputs
- [ ] `evaluate()` — executes graph in topological order
- [ ] Automatic intermediate texture allocation

**Graph structure (simplified):**
```rust
pub struct TextureGraph {
    nodes: Vec<Node>,
    connections: Vec<Connection>,
    output_size: (u32, u32),
}

pub struct Node {
    id: usize,
    operator: Box<dyn GpuOperator>,
    output: Option<TextureHandle>,
}

pub struct Connection {
    from_node: usize,
    to_node: usize,
    to_input: usize,
}

impl TextureGraph {
    pub fn evaluate(&mut self, ctx: &GpuContext) {
        let mut frame = GpuFrame::new(ctx.device);

        for node in self.nodes_in_topological_order() {
            let inputs = self.gather_inputs(node);
            let output = self.allocate_output(node, ctx);
            node.operator.execute(ctx, &mut frame, &inputs, &output);
            node.output = Some(output);
        }

        ctx.queue.submit([frame.finish()]);
    }
}
```

---

### Phase 4: Output to Disk

Read back final texture and save to file.

**Deliverables:**
- [ ] `OutputNode` — marks a texture for export
- [ ] `readback_texture()` — async GPU → CPU copy
- [ ] PNG export using `image` crate
- [ ] Float → u8 conversion with optional sRGB

**Readback flow:**
```rust
impl TextureGraph {
    pub fn evaluate_and_export(&mut self, width: u32, height: u32) {
        // Setup wgpu (headless, no window)
        let ctx = GpuContext::new_headless();

        self.output_size = (width, height);
        self.evaluate(&ctx);

        // Find output nodes and readback
        for node in self.output_nodes() {
            let pixels = ctx.readback_texture(&node.output);
            let path = node.output_path();

            // Convert Rgba16Float → Rgba8 with sRGB
            let rgba8 = convert_to_srgb_u8(&pixels);

            image::save_buffer(path, &rgba8, width, height, image::ColorType::Rgba8)?;
        }
    }
}
```

---

### Phase 5: Add More Operators

Expand operator library to make interesting textures.

**Deliverables:**
- [ ] `Levels` — black point, white point, gamma
- [ ] `GaussianBlur` — separable blur
- [ ] `Warp` — distort using another texture
- [ ] `NormalFromHeight` — generate normal map
- [ ] `Gradient` — linear/radial gradients

**Example graph after Phase 5:**
```rust
let graph = TextureGraph::new()
    // Base noise
    .node("noise", PerlinNoise::new().frequency(8.0).octaves(4))

    // Process height
    .node("levels", Levels::new().gamma(0.8))
    .connect("noise", "levels")

    // Generate normal
    .node("normal", NormalFromHeight::new().strength(1.0))
    .connect("levels", "normal")

    // Outputs
    .output("levels", "stone_height.png")
    .output("normal", "stone_normal.png");

graph.evaluate_and_export(2048, 2048);
```

---

## File Structure

```
flux/
├── solar-gpu/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── context.rs      # GpuContext
│       ├── frame.rs        # GpuFrame
│       ├── texture.rs      # TextureHandle
│       └── readback.rs     # GPU → CPU
│
├── flux-gpu/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── operator.rs     # GpuOperator trait
│       ├── graph.rs        # TextureGraph (PoC only)
│       ├── operators/
│       │   ├── mod.rs
│       │   ├── noise.rs    # PerlinNoise
│       │   ├── solid.rs    # SolidColor
│       │   ├── blend.rs    # Blend
│       │   ├── levels.rs   # Levels
│       │   ├── blur.rs     # GaussianBlur
│       │   └── normal.rs   # NormalFromHeight
│       └── shaders/
│           ├── perlin.wgsl
│           ├── blend.wgsl
│           ├── levels.wgsl
│           ├── blur.wgsl
│           └── normal.wgsl
│
└── examples/
    └── texture_generator/
        ├── Cargo.toml
        └── src/
            └── main.rs     # PoC demo
```

---

## Dependencies

```toml
# solar-gpu/Cargo.toml
[dependencies]
wgpu = "23"
pollster = "0.4"      # Blocking async for simplicity
bytemuck = "1.14"     # Safe casting for GPU buffers

# flux-gpu/Cargo.toml
[dependencies]
solar-gpu = { path = "../solar-gpu" }
flux-core = { path = "../flux-core" }

# examples/texture_generator/Cargo.toml
[dependencies]
solar-gpu = { path = "../../solar-gpu" }
flux-gpu = { path = "../../flux-gpu" }
image = "0.25"        # PNG export
```

---

## What's Deferred to Later

| Feature | Why deferred |
|---------|--------------|
| Resource pooling | PoC allocates fresh, optimize later |
| Dirty flag caching | PoC re-evaluates everything |
| CPU/GPU transparency | PoC is GPU-only |
| Integration with flux-graph | PoC has own simple graph |
| Windowed preview | PoC is headless, exports to disk |
| Editor UI | Out of scope for PoC |

---

## Milestones

1. **solar-gpu compiles** — Empty crate, wgpu dependency works
2. **First texture allocated** — Can create a GPU texture
3. **First shader runs** — Perlin noise fills a texture
4. **First PNG exported** — Readback works, file on disk
5. **Two-node graph works** — Noise → Levels → Output
6. **Demo graph complete** — Interesting procedural texture generated
