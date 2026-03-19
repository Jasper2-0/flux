# Three-Phase Evaluation for WGPU-Based GPU Operators

## Problem

GPU operators (noise, blur, blend, etc.) need fundamentally different evaluation from CPU operators. In the current single-pass `compute()` model, each GPU operator would individually: create buffers, upload data, dispatch a compute shader, wait for the GPU, and read back results. This is highly inefficient because:

1. **No batching** - Each GPU op submits and waits independently
2. **Excessive GPU synchronization** - CPU blocks on GPU for every operator
3. **No pipeline overlap** - Can't overlap uploads, dispatches, and readbacks
4. **Redundant buffer allocations** - No reuse of intermediate GPU textures

## Solution: Three-Phase Evaluation

Split evaluation into three phases that separate *what* to compute from *how* to execute it on the GPU:

### Phase 1: Prepare (CPU)
Walk the graph in topological order. For each GPU operator, call `prepare()` instead of `compute()`. This collects a `GpuTask` descriptor specifying the shader, input textures, output dimensions, and uniforms — but does **no** GPU work. CPU operators still run `compute()` normally.

### Phase 2: Execute (GPU)
Batch all `GpuTask`s into a single `wgpu::CommandEncoder`. Upload all inputs, dispatch all compute shaders, insert barriers between dependent tasks, and submit once. The GPU processes the entire chain without CPU round-trips.

### Phase 3: Readback (CPU)
Map output buffers back to CPU. Convert GPU results to `Value` types (e.g., `Value::FloatList` for texture data) and store in the graph's output cache so downstream CPU operators can consume them.

## Implementation Plan

### 1. Add `GpuOperator` trait to `flux-core` (no wgpu dependency)

**File: `flux-core/src/gpu_operator.rs`**

```rust
/// Descriptor for a GPU compute task — no GPU types, just data.
pub struct GpuTask {
    pub shader_id: &'static str,       // identifies which shader to use
    pub workgroup_size: [u32; 3],       // dispatch dimensions
    pub uniforms: Vec<u8>,              // packed uniform data
    pub input_bindings: Vec<GpuBinding>, // what to bind
    pub output_size: usize,             // output buffer size in bytes
}

pub enum GpuBinding {
    /// Read from a previous GPU task's output (by task index)
    GpuBuffer(usize),
    /// Upload CPU data to a buffer
    CpuData(Vec<u8>),
}

/// Trait for operators that run on the GPU.
/// Extends Operator — the `compute()` method serves as CPU fallback.
pub trait GpuOperator: Operator {
    /// Phase 1: Describe what GPU work to do, without doing it.
    fn prepare(&self, ctx: &EvalContext, get_input: InputResolver) -> GpuTask;

    /// Phase 3: Convert raw GPU output bytes back to Value.
    fn readback(&mut self, output_data: &[u8]);
}
```

This keeps `flux-core` free of wgpu dependencies. The trait uses only plain data types.

### 2. Create `flux-gpu` crate

**New crate: `flux-gpu/`**

Dependencies: `wgpu`, `flux-core`, `flux-graph`

Contains:
- **`GpuContext`** — Wraps `wgpu::Device`, `Queue`, shader module cache, buffer pool. Stored as an `EvalContext` extension.
- **`GpuEvaluator`** — The three-phase evaluation engine that plugs into `Graph::evaluate_with_callback()`.
- **Shader registry** — Maps `shader_id` strings to compiled `wgpu::ShaderModule`s.
- **Buffer pool** — Reuses GPU buffers across frames to avoid allocation churn.

### 3. Implement `GpuEvaluator` in `flux-gpu`

**File: `flux-gpu/src/evaluator.rs`**

```rust
pub struct GpuEvaluator {
    gpu_ctx: Arc<GpuContext>,
}

impl GpuEvaluator {
    /// Run three-phase evaluation on a graph.
    pub fn evaluate(
        &self,
        graph: &mut Graph,
        output_node: Id,
        output_index: usize,
        ctx: &EvalContext,
    ) -> Result<Value, GraphError> {
        // Phase 1: Prepare — collect GpuTasks
        let mut gpu_tasks: Vec<(Id, GpuTask)> = Vec::new();

        graph.evaluate_with_callback(output_node, output_index, ctx, |op, id, ctx, get_input| {
            // Try to downcast to GpuOperator
            if let Some(gpu_op) = op.as_any().downcast_ref::<dyn GpuOperator>() {
                let task = gpu_op.prepare(ctx, &get_input);
                gpu_tasks.push((id, task));
                false // don't call compute() yet — we'll set outputs in phase 3
            } else {
                false // let default compute() handle CPU ops
            }
        })?;

        if gpu_tasks.is_empty() {
            return graph.evaluate(output_node, output_index, ctx);
        }

        // Phase 2: Execute — batch GPU work
        let results = self.execute_gpu_batch(&gpu_tasks);

        // Phase 3: Readback — convert results and update cache
        for ((node_id, _), output_data) in gpu_tasks.iter().zip(results.iter()) {
            if let Some(op) = graph.get_mut(*node_id) {
                if let Some(gpu_op) = op.as_any_mut().downcast_mut::<dyn GpuOperator>() {
                    gpu_op.readback(output_data);
                }
            }
        }

        graph.get_cached_output(output_node, output_index, ctx.call_context)
    }

    fn execute_gpu_batch(&self, tasks: &[(Id, GpuTask)]) -> Vec<Vec<u8>> {
        let device = &self.gpu_ctx.device;
        let queue = &self.gpu_ctx.queue;
        let mut encoder = device.create_command_encoder(&Default::default());

        // Create buffers, bind groups, dispatch all tasks
        // ... (wgpu implementation details)

        queue.submit(std::iter::once(encoder.finish()));

        // Map and read output buffers
        // ...
    }
}
```

### 4. Add example GPU operators

**File: `flux-gpu/src/operators/noise.rs`** — Perlin/Simplex noise computed on GPU
**File: `flux-gpu/src/operators/blur.rs`** — Gaussian blur
**File: `flux-gpu/src/operators/blend.rs`** — Texture blending

Each implements both `Operator` (CPU fallback) and `GpuOperator` (GPU path).

### 5. Wire into workspace

- Add `flux-gpu` to workspace `Cargo.toml`
- Add `downcast-rs` to `flux-core` workspace deps (already present)
- Add example `33_gpu_three_phase.rs` demonstrating the pipeline

## Key Design Decisions

1. **`flux-core` stays GPU-agnostic** — `GpuOperator` trait uses only plain Rust types. The wgpu dependency lives only in `flux-gpu`.

2. **Leverages existing `evaluate_with_callback()`** — The graph already has this hook (line 1321 of graph.rs) for exactly this purpose. No changes to the core evaluation loop.

3. **CPU fallback via `compute()`** — Every `GpuOperator` still implements `Operator::compute()` as a CPU fallback, so graphs work without a GPU.

4. **Extensions for GPU context** — The `EvalContext::extensions` system (already in flux-core) carries `GpuContext` through evaluation without any new coupling.

5. **Buffer pool for performance** — GPU buffers are reused across frames via a pool, avoiding the main perf killer in naive GPU usage.

## File Changes Summary

| Action | File | Description |
|--------|------|-------------|
| Create | `flux-core/src/gpu_operator.rs` | `GpuOperator` trait + `GpuTask` types |
| Edit | `flux-core/src/lib.rs` | Export new module |
| Create | `flux-gpu/Cargo.toml` | New crate with wgpu dep |
| Create | `flux-gpu/src/lib.rs` | Crate root |
| Create | `flux-gpu/src/context.rs` | `GpuContext` (device, queue, pool) |
| Create | `flux-gpu/src/evaluator.rs` | `GpuEvaluator` three-phase engine |
| Create | `flux-gpu/src/buffer_pool.rs` | GPU buffer reuse |
| Create | `flux-gpu/src/shader_registry.rs` | Shader compilation cache |
| Create | `flux-gpu/src/operators/mod.rs` | GPU operator implementations |
| Create | `flux-gpu/src/operators/noise.rs` | GPU noise operator |
| Edit | `Cargo.toml` | Add flux-gpu to workspace |
| Create | `examples/33_gpu_three_phase.rs` | Demo example |
