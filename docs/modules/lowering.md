# Lowering Pipeline

[← Back to Module Reference](../module_reference.html) | [← Documentation Home](../index.html)

> Lowering turns high-level syntax into a simplified HIR ready for SSA.

## Overview

`src/lower` transforms the AST into a high-level intermediate representation
(HIR). This IR strips surface sugar, standardises control flow, and prepares the
program for SSA lowering.

## Key Structures

- **HModule** – Collects lowered items while preserving module paths for
  downstream passes.
- **HItem / HFunction** – Represent lowered functions with parameter lists and
  bodies ready for SSA conversion.
- **HBlock / HStmt** – Model structured blocks with `let`, expression, and return
  statements after desugaring control flow.
- **HExpr** – Encodes literals, paths, record literals, method-call desugarings,
  and binary operations in a uniform format.
- **HFuncRef** – Distinguishes between direct calls and methods lowered to
  receiver-first functions.

## Transformation Flow

1. `lower_module` iterates over AST items and lowers supported constructs while
   preserving module identity.
2. `lower_function` copies parameter metadata and lowers bodies using expression
   helpers so later phases no longer depend on AST details.
3. `lower_block` / `lower_expr` desugar method calls, indexing, assignments,
   concurrency primitives, and effect helpers into canonical call forms.

## Integration Notes

- Lowered functions feed directly into SSA lowering without re-traversing the
  AST.
- CLI mode `--lower` prints `hir_to_string` output for debugging, ensuring
  desugarings behave as expected.
- Regression tests assert on lowering behaviour to catch regressions as new
  syntax or semantics land.

## Roadmap Alignment

- **Phase 2** – Provides the high-level IR required before SSA work begins and
  isolates desugaring logic from later optimisation passes.
- **Phase 3** – Will grow to cover additional item kinds (types, constants) so
  SSA lowering receives a richer view of each module.

## Next Steps

- Introduce explicit control-flow nodes rather than encoding branches as helper
  calls.
- Track source spans through lowering to improve downstream diagnostics.
- Provide hooks for incremental or cached lowering in anticipation of IDE
  integrations.

---

## Related Modules

- **[Syntax Front-End](syntax.html)** — Provides the AST input
- **[Semantic Analysis](semantics.html)** — Supplies resolved metadata
- **[SSA IR](ir.html)** — Consumes the HIR output

[← Back to Module Reference](../module_reference.html) | [← Documentation Home](../index.html)
