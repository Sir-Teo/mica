# SSA Intermediate Representation

## Scope

The SSA-oriented IR defined in `src/ir/mod.rs` lowers the high-level HIR into
basic blocks, instructions, and typed values. It forms the backbone of Phase 2
and Phase 3 roadmap milestones focused on optimization and code generation.

## Core Types

| Type | Purpose |
| --- | --- |
| `Module` | Groups lowered functions for a module and preserves the module path for backend consumers.【F:src/ir/mod.rs†L6-L99】 |
| `Function` | Holds parameter metadata, return type, and the sequence of basic blocks produced by the SSA lowerer.【F:src/ir/mod.rs†L12-L149】 |
| `BasicBlock` | Represents a control-flow node with instructions and a terminator (currently return).【F:src/ir/mod.rs†L27-L63】 |
| `Instruction` | Encodes SSA values with IDs, inferred types, and instruction kinds such as literals, binary ops, calls, and records.【F:src/ir/mod.rs†L34-L58】 |
| `FuncRef` | Distinguishes direct function calls from desugared method calls as they flow through SSA lowering.【F:src/ir/mod.rs†L65-L69】 |
| `Type` | Tracks coarse static types inferred during lowering to aid diagnostics and future optimization passes.【F:src/ir/mod.rs†L77-L85】 |

## Lowering Process

- `lower_module` walks the HIR and lowers each `HFunction` into a SSA `Function`.
  Unsupported items are skipped, mirroring the Phase 2 focus on executable code
  paths.【F:src/ir/mod.rs†L87-L108】【F:docs/roadmap/compiler.md†L126-L170】
- `FunctionLower` maintains scope stacks, allocates SSA value IDs, and appends
  instructions as it traverses statements and expressions.【F:src/ir/mod.rs†L110-L200】
- Literal emission, binary operations, calls, records, and returns are converted
  into typed instructions, with placeholder `Type::Unknown` until richer typing
  arrives in later phases.【F:src/ir/mod.rs†L136-L323】【F:docs/roadmap/compiler.md†L170-L215】

## Integration Highlights

1. Consumes the HIR structures from the lowering stage, ensuring desugared
   constructs map cleanly onto SSA building blocks.【F:src/ir/mod.rs†L3-L107】【F:src/lower/mod.rs†L3-L199】
2. Provides the foundation for future optimization passes, register allocation,
   and backend code generation described in the roadmap’s Phase 3 milestones.【F:docs/roadmap/compiler.md†L170-L215】
3. Tests under `src/tests` verify SSA pretty-printing and instruction sequences,
   preventing regressions as the IR evolves.【F:src/tests/ir_tests.rs†L1-L75】

## Roadmap Alignment

- **Phase 2:** Establishes SSA representation and lowering, enabling forthcoming
  control-flow restructuring and analysis passes.【F:docs/roadmap/compiler.md†L126-L170】
- **Phase 3:** Serves as the input to code generation, optimization, and runtime
  integration tasks outlined for the compiler backend.【F:docs/roadmap/compiler.md†L170-L215】
- **Tooling:** SSA dumps and diagnostics will be surfaced through CLI extensions
  and IDE tooling to aid debugging, per the tooling roadmap.【F:docs/roadmap/tooling.md†L1-L60】

## Next Steps

- Expand terminators to include branches and structured control flow as SSA
  support matures.
- Track precise types by integrating with the planned Hindley–Milner inference
  and capability analysis.
- Emit machine-readable dumps (JSON, DOT) to integrate with visualization tools
  and future IDE plugins.
