# Semantic Analysis

> Resolution and effect checking turn parsed syntax into meaningful program
> structure with actionable diagnostics.

## Overview

Semantic analysis builds on the syntax front-end. It resolves names, tracks
capability usage, and performs lightweight type/effect checks before lowering.
All functionality lives under `src/semantics`.

## Name Resolution

- Builds module graphs and symbol tables for single or multi-module workspaces.
- Records imports, type definitions, capabilities, and resolved identifiers in a
  `Resolved` snapshot.
- Tracks which functions and type aliases require capabilities so later passes
  can validate usage.

## Type and Effect Checking

- Walks each module to collect function signatures and algebraic data types.
- Enforces capability usage, return correctness, and effect annotations using the
  resolver metadata.
- Performs exhaustiveness checking on `match` expressions to highlight missing
  variants.
- Aggregates diagnostics without panicking so the CLI and tests can surface
  actionable warnings.

## Integration Notes

1. Resolver output feeds the checker with symbol metadata and capability context.
2. CLI modes (`--resolve`, `--check`) expose semantic information for manual and
   automated validation.
3. Lowering and IR generation consume resolved data to attach stable identifiers
   and effect information.

## Roadmap Alignment

- **Phase 1** – Implements the baseline resolution and effect checking described
  in the roadmap, paving the way for inference and borrow checking.
- **Phase 2** – Structured outputs provide the data required for SSA lowering and
  optimisation passes.
- **Tooling** – Rich metadata enables symbol search, documentation generation,
  and eventual language server integrations.

## Next Steps

- Extend capability checks with region and lifetime reasoning to support upcoming
  borrow-checking milestones.
- Emit machine-readable diagnostics to integrate with the planned LSP.
- Add caching and incremental recomputation hooks to scale to large workspaces.
