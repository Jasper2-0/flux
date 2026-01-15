# GPU Resource Pool Research Assignment

## Context

Flux is adding GPU support via wgpu. We need to design a resource pool that:
- Manages GPU resources (buffers, textures, etc.)
- Integrates with Flux's dirty flag system for cache invalidation
- Supports multi-frame resource lifetimes
- Batches commands during update() for submission in draw()

Research how existing creative coding frameworks handle these concerns.

---

## Research Questions

### 1. Resource Types Managed

What GPU resources does the framework's pool/manager handle?

- [ ] Buffers (storage, uniform, vertex, index)
- [ ] Textures (2D, 3D, cube maps, render targets, depth)
- [ ] Samplers
- [ ] Bind groups / descriptor sets
- [ ] Pipelines (compute, render)
- [ ] Command buffers/encoders
- [ ] Other?

**Questions:**
- Are all resource types managed uniformly, or do different types have different strategies?
- Are pipelines cached/pooled, or created on-demand?

---

### 2. Handle Design

How does the framework represent references to GPU resources?

**Patterns to look for:**

```
A) Generational index
   - Index + generation counter
   - Validity check: compare generations
   - Used by: slotmap, thunderdome, bevy

B) Arc/Rc based
   - Reference counted smart pointer
   - Automatic cleanup when count hits zero
   - Simpler but less control over timing

C) Raw ID / opaque handle
   - Just an integer, pool tracks validity separately
   - Manual lifetime management

D) Typed wrapper around backend handle
   - Thin wrapper around wgpu::Buffer, wgpu::Texture, etc.
   - Framework adds metadata
```

**Questions:**
- How do you check if a handle is still valid?
- Can handles be cloned/shared between operators?
- What happens when you use an invalid handle?

---

### 3. Allocation Strategy

How does the framework allocate GPU memory?

**Patterns to look for:**

```
A) On-demand allocation
   - Each request creates new GPU resource
   - Simple, but potential fragmentation

B) Pool by size bucket
   - Pre-allocated pools: 64KB, 256KB, 1MB, etc.
   - Request gets next-largest bucket
   - Fast allocation, some waste

C) Suballocation
   - Large GPU buffer, hand out slices
   - Offset + length instead of separate allocations
   - Complex but efficient

D) Ring buffer
   - Circular buffer for transient per-frame data
   - Head advances each frame
   - Great for uniform data that changes every frame

E) Buddy allocator / free list
   - Classic memory allocator strategies adapted for GPU
```

**Questions:**
- Does the framework use different strategies for different resource types?
- How is alignment handled?
- Is there a distinction between persistent and transient allocations?

---

### 4. Invalidation and Cache Integration

How does the framework know when a cached GPU resource is stale?

**Patterns to look for:**

```
A) Explicit release
   - User/operator calls release(handle)
   - Simple but error-prone

B) Dependency tracking
   - Resource knows its dependencies
   - Invalidates when dependencies change
   - Like a build system (make, bazel)

C) Dirty flag / reactive
   - Resource linked to dirty flag
   - Pool checks flag before reuse
   - Flux's current model

D) Content hashing
   - Hash inputs, cache by hash
   - Automatic deduplication
   - Used by some shader/pipeline caches

E) Frame tagging
   - Resources tagged with last-used frame
   - Evict after N frames unused
```

**Questions:**
- Is invalidation push (notify pool) or pull (pool checks)?
- Can partial invalidation happen (e.g., just the mip levels)?
- How does invalidation cascade to dependent resources?

---

### 5. Reclamation Timing

When does the framework actually free GPU memory?

**Patterns to look for:**

```
A) Immediate
   - Free as soon as invalidated
   - Simple, but may free resources still in GPU queue

B) Deferred to frame end
   - Collect invalidated handles, free after submit
   - Ensures GPU is done with resources

C) Deferred N frames
   - Free after N frames (typically 2-3)
   - Accounts for GPU pipeline depth

D) On memory pressure
   - Only free when allocation fails or threshold hit
   - Maximizes reuse

E) Explicit flush
   - User calls pool.cleanup() or pool.gc()
```

**Questions:**
- How does the framework handle resources still referenced by in-flight GPU commands?
- Is there a memory budget / limit?
- Can you force immediate cleanup?

---

### 6. Thread Safety

How does the framework handle multi-threaded access to the pool?

**Patterns to look for:**

```
A) Single-threaded only
   - Pool accessed from one thread
   - Simple, works for many creative coding use cases

B) Mutex-protected
   - Lock around pool operations
   - Safe but potential contention

C) Per-thread pools
   - Each thread has own pool, merge at frame end
   - Used by some job systems

D) Lock-free structures
   - Concurrent queues, atomics
   - Complex but scalable

E) Command recording thread-local, submission single-threaded
   - Multiple threads record, one thread submits
   - Common pattern in Vulkan/wgpu
```

**Questions:**
- Can resources be created from multiple threads?
- Is command recording thread-safe?
- How are resources shared between threads?

---

### 7. Command Batching

How does the framework batch GPU commands?

**Patterns to look for:**

```
A) Immediate submission
   - Each operation submits immediately
   - Simple but inefficient

B) Single command buffer per frame
   - All operations record to one buffer
   - Submit once at end of frame

C) Multiple command buffers
   - Separate buffers for compute vs render
   - Or per-pass separation

D) Render graph
   - Declare passes and dependencies
   - Framework optimizes ordering and barriers
   - Used by bevy, modern engines

E) Deferred command list
   - Record high-level commands
   - Compile to GPU commands at submit time
```

**Questions:**
- How are dependencies between commands handled?
- Are barriers/synchronization inserted automatically?
- Can you have multiple command streams (e.g., async compute)?

---

## Frameworks to Study

### Rust Ecosystem
- [ ] **Bevy** - ECS game engine, render graph, asset system
- [ ] **Nannou** - Creative coding, wgpu-based
- [ ] **wgpu examples** - Low-level patterns
- [ ] **rend3** - Rendering library built on wgpu

### Other Languages
- [ ] **three.js** (JS) - WebGL/WebGPU renderer
- [ ] **Processing/p5.js** - Creative coding, simpler model
- [ ] **openFrameworks** (C++) - Creative coding, OpenGL
- [ ] **Cinder** (C++) - Creative coding, lower-level
- [ ] **TouchDesigner** - Node-based, GPU-heavy
- [ ] **Notch** - Real-time graphics, node-based

### Game Engines
- [ ] **Godot** - Open source, recently added Vulkan
- [ ] **Unreal** - RHI abstraction, render graph
- [ ] **Unity** - SRP, command buffers

---

## Deliverables

After researching, document:

1. **Comparison table** - How each framework answers each question
2. **Recommended approach** - Which patterns fit Flux's architecture
3. **Open questions** - Things that need prototyping to decide
4. **Reference code** - Links to relevant source files in studied frameworks

---

## Notes

(Add notes during research here)
