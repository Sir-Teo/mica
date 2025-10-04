# SSA Intermediate Representation

> The SSA IR bridges desugared syntax and backend code generation.

## Overview

`src/ir` defines a typed single static assignment (SSA) representation. It lowers
HIR into functions, basic blocks, and instructions while preserving type and
capability metadata. The module doubles as documentation for backend contracts.

## Core Concepts

- **Module** – Owns lowered functions plus shared type and effect tables so
  backends reuse canonical metadata.
- **Function** – Stores parameters, return metadata, basic blocks, and required
  capabilities.
- **Basic block** – Groups SSA instructions with a single terminator.
- **Instruction** – Encodes literals, binary operations, calls, record builders,
  and resolved paths with explicit type IDs.
- **Type/Effect tables** – Intern structural types and effect names so large
  modules stay cheap to clone and inspect.

## Lowering Flow

1. `lower_module` walks the HIR, interns referenced types/effects, and lowers
   each function.
2. `FunctionLower` allocates SSA value IDs, attaches type metadata, and threads
   effect information through returns.
3. Literals, operations, calls, records, and returns emit typed instructions,
   defaulting to `unknown` placeholders until later inference phases refine them.

## Integration Notes

- Consumes HIR emitted by the lowering stage, ensuring desugared constructs map
  cleanly onto SSA building blocks.
- Provides canonical type/effect registries that textual, LLVM, and native
  backends consume without re-resolving AST nodes.
- Regression tests exercise IR typing, effect rows, and backend output to prevent
  regressions as the IR evolves.

## Roadmap Alignment

- **Phase 2** – Establishes the SSA baseline required for control-flow analysis
  and optimisation work.
- **Phase 3** – Feeds code generation, optimisation, and runtime integration
  tasks in the backend roadmap.
- **Tooling** – SSA dumps and diagnostics are exposed through the CLI for future
  IDE integrations.

## Next Steps

- Introduce additional terminators (branches, structured control flow) as SSA
  support matures.
- Integrate precise types via the planned inference work and capability analysis.
- Emit machine-readable dumps (JSON, DOT) for visualisation tools and editor
  plugins.
