# Flux Geometry System

Procedural geometry creation and manipulation for real-time graphics.

## Overview

The geometry system enables Flux to create, modify, and render 3D geometry through the dataflow graph. Geometry flows as a first-class `Value` type alongside floats, vectors, and colors.

**Key capabilities:**
- Procedural mesh generation (boxes, spheres, grids)
- Per-element attribute storage (positions, normals, colors, UVs)
- Transform and combine operations
- GPU-ready output via existing `MeshHandle` system

## Design Principles

### 1. Real-Time First

Flux targets 60fps+ rendering. Every design decision optimizes for this:

- **Dirty-tracked evaluation** — Geometry only regenerates when inputs change
- **GPU upload on demand** — CPU geometry converts to GPU buffers only when modified
- **Static geometry is common** — Most meshes generate once, render many frames

### 2. Geometry as Bundle

Geometry flows through the graph as a single opaque value, not decomposed into separate position/normal/index wires:

```
[BoxGeometry] ──▶ Geometry ──▶ [Transform] ──▶ Geometry ──▶ [UploadGeometry] ──▶ MeshHandle
```

This keeps graphs clean and enables operations that need full mesh context (transforms, merges, subdivision).

### 3. Bridge Operators for Access

When you need raw data (positions as `Vec3List`, indices as `IntList`), explicit bridge operators extract it:

```
[Geometry] ──▶ [GetPositions] ──▶ Vec3List
[Geometry] ──▶ [GetNormals] ──▶ Vec3List
[Geometry] ──▶ [GetAttribute "uv"] ──▶ Vec2List
```

This makes data flow explicit and avoids hidden precedence rules.

### 4. Four Attribute Levels

Each geometry element can carry attributes at four levels:

| Level | Scope | Example Attributes |
|-------|-------|-------------------|
| **Point** | Per unique position | `P` (position), `N` (normal), `Cd` (color) |
| **Vertex** | Per corner per face | `uv`, split normals |
| **Primitive** | Per face | material ID, face normal |
| **Detail** | Whole geometry | bounding box, point count |

This matches industry-standard geometry representations and enables workflows like UV seams (different UVs at same position) and per-face materials.

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        CPU Domain                                │
│                                                                  │
│  [Generator] ──▶ Geometry ──▶ [Modifier] ──▶ Geometry           │
│       │                            │                             │
│       └────────────────────────────┴──────────┐                 │
│                                               ▼                  │
│                                    [UploadGeometry]             │
│                                               │                  │
├───────────────────────────────────────────────┼──────────────────┤
│                        GPU Domain             ▼                  │
│                                                                  │
│                                         MeshHandle              │
│                                               │                  │
│                                      [Render Pipeline]          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Integration with Existing Types

Geometry builds on Flux's existing type system:

| Existing Type | Geometry Use |
|---------------|--------------|
| `Vec3List` | Positions, normals, colors |
| `Vec2List` | UVs (new addition) |
| `IntList` | Face indices, point indices |
| `FloatList` | Scalar attributes (weights, ages) |
| `Matrix4` | Transforms |
| `MeshHandle` | GPU-uploaded geometry for rendering |

### Dirty Flag Integration

Geometry operators participate in Flux's lazy evaluation:

1. `BoxGeometry` generates mesh, marks output clean
2. `UploadGeometry` creates GPU buffers, marks clean
3. Frame N+1: nothing changed → no work
4. User changes box size → `BoxGeometry` dirty → regenerates → `UploadGeometry` dirty → re-uploads

This ensures geometry work only happens when needed.

## Use Cases

### Static Procedural Geometry

Generate once, render many frames:

```
[BoxGeometry] ──▶ [Transform] ──▶ [UploadGeometry] ──▶ [Render]
     │
     └── size: [1, 2, 1]  (constant)
```

### Dynamic Geometry

Regenerate per-frame (e.g., audio-reactive):

```
[GridGeometry] ──▶ [DisplaceByNoise] ──▶ [UploadGeometry] ──▶ [Render]
     │                    │
     └── rows: 100        └── [AudioAmplitude] (changes every frame)
```

### Instancing

Render thousands of copies efficiently. `Arc<Geometry>` enables zero-cost sharing:

```
[SphereGeometry] ──▶ [CopyToPoints] ──▶ [Render]
                          │
[ScatterPoints] ──────────┘

(All 10,000 copies share the same Arc<Geometry> - no memory duplication)
```

## Document Index

- [types.md](types.md) — Core type definitions (`Geometry`, `AttributeTable`)
- [operators.md](operators.md) — Operator catalog (generators, modifiers, bridge)
- [implementation.md](implementation.md) — Phased implementation roadmap
