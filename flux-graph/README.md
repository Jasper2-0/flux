# flux-graph

Graph execution engine, symbol system, and serialization for Flux.

## Features

- **Graph structure** - Connect operators with type-checked connections
- **Topological evaluation** - Automatic dependency ordering
- **Symbol system** - Reusable operator definitions with instances
- **Animation** - Keyframe curves with multiple interpolation modes
- **Undo/Redo** - Command-based history for editor integration
- **Serialization** - JSON format for graphs, symbols, and projects

## Key Types

| Type | Description |
|------|-------------|
| `Graph` | Main graph structure for connecting operators |
| `Connection` | Represents a connection between two ports |
| `AssociatedGraph` | Graph wrapper with external ID management |
| `CompositeOp` | Nested graph operator (graph-within-graph) |
| `UndoRedoStack` | Command history for undo/redo support |
| `CompiledGraph` | Optimized graph for repeated evaluation |

## Quick Example

```rust
use flux_graph::{Graph, Connection};
use flux_operators::{ConstantOp, AddOp};
use flux_core::EvalContext;

// Create a graph
let mut graph = Graph::new();

// Add operators
let a = graph.add(Box::new(ConstantOp::new(5.0)));
let b = graph.add(Box::new(ConstantOp::new(3.0)));
let add = graph.add(Box::new(AddOp::new()));

// Connect: a + b
graph.connect(a, 0, add, 0)?; // a.out[0] -> add.in[0]
graph.connect(b, 0, add, 1)?; // b.out[0] -> add.in[1]

// Evaluate
let ctx = EvalContext::new();
let result = graph.evaluate(add, 0, &ctx)?;
println!("5 + 3 = {}", result); // "5 + 3 = 8"
```

## Modules

| Module | Description |
|--------|-------------|
| `graph` | Core Graph type, Connection, GraphStats |
| `symbol` | Symbol definitions and instance management |
| `animation` | Keyframe animation with interpolation |
| `composite` | CompositeOp for nested graphs |
| `serialization` | JSON serialization/deserialization |
| `commands` | AddNode, Connect, Disconnect commands |
| `undo` | UndoRedoStack for history management |
| `compiler` | Graph compilation for optimized evaluation |
| `bypass` | Node bypass state management |

## File Formats

| Extension | Description |
|-----------|-------------|
| `.rsym` | Symbol definitions (reusable operator graphs) |
| `.rgraph` | Complete graph with all operators and connections |
| `.rproj` | Project file with multiple graphs and resources |

## Animation System

```rust
use flux_graph::animation::{Animation, Keyframe, InterpolationMode};

// Create an animation curve
let mut anim = Animation::new();
anim.add_keyframe(Keyframe::new(0.0, 0.0, InterpolationMode::Linear));
anim.add_keyframe(Keyframe::new(1.0, 100.0, InterpolationMode::EaseInOut));

// Sample at any time
let value = anim.sample(0.5); // 50.0 (linear interpolation)
```

## Command System (Undo/Redo)

```rust
use flux_graph::{Graph, UndoRedoStack};
use flux_graph::commands::{AddNodeCommand, ConnectCommand};

let mut graph = Graph::new();
let mut history = UndoRedoStack::new();

// Execute commands through the stack
let cmd = AddNodeCommand::new(Box::new(ConstantOp::new(42.0)));
let node_id = history.execute(&mut graph, cmd)?;

// Undo/redo
history.undo(&mut graph)?;
history.redo(&mut graph)?;
```

## See Also

- [flux-core](../flux-core/) - Foundation types (Value, Operator)
- [flux-operators](../flux-operators/) - Built-in operators
- [Evaluation Flow](../docs/02-architecture/evaluation-flow.md) - How data moves through graphs
- [Symbol/Instance](../docs/02-architecture/symbol-instance.md) - Definition vs runtime state
