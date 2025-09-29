# SSA Intermediate Representation

## Scope

The SSA-oriented IR defined in `src/ir/mod.rs` lowers the high-level HIR into
basic blocks, instructions, and typed values. It forms the backbone of Phase 2
and Phase 3 roadmap milestones focused on optimization and code generation.

## Core Types

| Type | Purpose |
| --- | --- |
| `Module` | Owns lowered functions alongside shared type/effect tables so backends see a self-contained package of IR plus metadata.【F:src/ir/mod.rs†L8-L169】 |
| `Function` | Holds parameters, `TypeId` return metadata, basic blocks, and effect identifiers describing capability requirements.【F:src/ir/mod.rs†L17-L225】 |
| `BasicBlock` | Represents a control-flow node with instructions and a terminator (currently return).【F:src/ir/mod.rs†L32-L68】 |
| `Instruction` | Encodes SSA values with IDs, referenced types, and instruction kinds such as literals, binary ops, calls, and records.【F:src/ir/mod.rs†L39-L63】【F:src/ir/mod.rs†L407-L416】 |
| `TypeTable` & `TypeId` | Intern and reuse structural types so large modules share canonical IDs while keeping lookups O(1).【F:src/ir/mod.rs†L100-L170】【F:src/ir/mod.rs†L538-L584】 |
| `EffectTable` & `EffectId` | Deduplicate effect names, allowing effect rows to scale linearly with references instead of strings.【F:src/ir/mod.rs†L106-L170】【F:src/ir/mod.rs†L586-L600】 |

## Lowering Process

- `lower_module` walks the HIR, interns every referenced type/effect, and then
  lowers each function so the resulting module is fully self-contained.【F:src/ir/mod.rs†L112-L170】
- `FunctionLower` maintains scope stacks, allocates SSA value IDs, attaches
  `TypeId`s to every SSA value, and appends instructions while threading effect
  metadata through returns.【F:src/ir/mod.rs†L173-L463】
- Literal emission, binary operations, calls, records, and tail returns produce
  typed instructions, defaulting to the shared `unknown` slot until inference can
  refine them in later phases.【F:src/ir/mod.rs†L294-L416】【F:src/ir/mod.rs†L443-L455】

## Integration Highlights

1. Consumes the HIR structures from the lowering stage, ensuring desugared
   constructs map cleanly onto SSA building blocks.【F:src/ir/mod.rs†L3-L399】【F:src/lower/mod.rs†L3-L320】
2. Ships canonical type and effect registries that backend integrations (text,
   LLVM, WASM) can query without re-resolving AST nodes.【F:src/ir/mod.rs†L498-L600】【F:src/backend/text.rs†L1-L129】
3. Tests under `src/tests` exercise IR typing, effect rows, and backend output,
   preventing regressions as the IR evolves.【F:src/tests/ir_tests.rs†L1-L132】【F:src/tests/backend_tests.rs†L1-L55】

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
