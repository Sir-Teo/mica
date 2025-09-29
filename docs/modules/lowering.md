# Lowering Pipeline

## Scope

The lowering stage (`src/lower/mod.rs`) transforms the high-level abstract syntax
into a simplified high-level intermediate representation (HIR). This IR removes
surface sugar, standardizes control flow, and prepares the program for SSA-based
lowering in the next stage of the roadmap.

## Data Structures

| Structure | Purpose |
| --- | --- |
| `HModule` | Collects lowered items for a module, preserving the module path for downstream passes.【F:src/lower/mod.rs†L3-L71】 |
| `HItem` | Enumerates the lowered item kinds currently supported (functions).【F:src/lower/mod.rs†L9-L13】 |
| `HFunction` | Stores function names, parameter lists, and lowered bodies for SSA conversion.【F:src/lower/mod.rs†L14-L80】 |
| `HBlock` / `HStmt` | Represent structured blocks with `let`, expression, and return statements after desugaring control flow constructs.【F:src/lower/mod.rs†L21-L96】 |
| `HExpr` | Captures literals, paths, method-call desugarings, record literals, and binary operations in a uniform format.【F:src/lower/mod.rs†L33-L199】 |
| `HFuncRef` | Distinguishes between direct function calls and method calls desugared into first-argument receivers.【F:src/lower/mod.rs†L54-L119】 |

## Transformation Responsibilities

- `lower_module` iterates over AST items and lowers functions, filtering out
  unsupported constructs while preserving module identity.【F:src/lower/mod.rs†L60-L71】
- `lower_function` copies parameter names and lowers the body using expression
  helpers, ensuring the resulting IR no longer depends on AST-specific details.【F:src/lower/mod.rs†L73-L120】
- `lower_block` and `lower_expr` recursively desugar method calls, indexing,
  assignment, concurrency primitives (`spawn`, `await`, channels), and effect
  helpers into canonical call forms.【F:src/lower/mod.rs†L82-L199】

## Integration Highlights

1. Lowered functions feed directly into the SSA lowering stage to emit basic
   blocks and instructions without re-traversing the original AST.【F:src/ir/mod.rs†L1-L208】
2. The CLI’s `--lower` mode prints `hir_to_string` output for debugging, helping
   validate that desugarings behave as expected across roadmap milestones.【F:src/main.rs†L176-L181】【F:src/lower/mod.rs†L200-L292】
3. Tests under `src/tests` assert on lowering behavior to catch regressions as
   new syntax or semantic rules are introduced.【F:src/tests/lowering_tests.rs†L1-L120】

## Roadmap Alignment

- **Phase 2:** The roadmap calls for establishing a high-level IR prior to SSA;
  this module already fulfills that requirement and isolates desugaring logic
  from later optimization passes.【F:docs/roadmap/compiler.md†L126-L170】
- **Phase 3:** As backend work begins, extend `HItem` to cover additional item
  kinds (e.g., type declarations, global constants) so SSA lowering receives a
  richer view of the module.【F:docs/roadmap/compiler.md†L170-L215】

## Next Steps

- Introduce explicit control-flow constructs in HIR (conditional, loop nodes)
  rather than encoding them via synthetic method calls, easing SSA generation.
- Track source spans through lowering to enable better diagnostics downstream.
- Provide hooks for incremental or cached lowering in anticipation of IDE
  integrations.
