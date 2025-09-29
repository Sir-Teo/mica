# Semantic Analysis

## Scope

Semantic analysis encompasses name resolution (`src/semantics/resolve`) and the
lightweight type/effect checker (`src/semantics/check.rs`). These passes sit on
Phase 1 of the compiler roadmap, building upon the syntax front-end to surface
language understanding and actionable diagnostics.

## Name Resolution (`resolve`)

- `resolve_module` constructs a `ModuleGraph`, gathers symbol tables, and drives
  the `Resolver` to produce a fully-populated `Resolved` structure for a single
  module.【F:src/semantics/resolve/mod.rs†L17-L28】
- `resolve_modules` supports multi-module workspaces by iterating through all
  collected modules, aligning with roadmap goals for packages and the standard
  library.【F:src/semantics/resolve/mod.rs†L30-L60】【F:docs/roadmap/ecosystem.md†L1-L78】
- `Resolved` captures module paths, ADT variants, imports, capabilities, resolved
  symbols, and diagnostics—data that powers the CLI `--resolve` mode and future
  language server features.【F:src/semantics/resolve/data.rs†L3-L106】【F:docs/roadmap/tooling.md†L1-L60】
- Capability binding tracking records which functions and type aliases require a
  capability, paving the way for effect checking and borrow discipline described
  in the roadmap.【F:src/semantics/resolve/data.rs†L75-L99】【F:docs/roadmap/compiler.md†L97-L125】

## Type and Effect Checking (`check.rs`)

- `check_module` instantiates a `Checker` that collects function signatures and
  ADT variants before walking every function body.【F:src/semantics/check.rs†L15-L194】
- Exhaustiveness analysis inspects `match` expressions to report missing variants
  when no wildcard arm is provided, surfacing user-friendly diagnostics.【F:src/semantics/check.rs†L23-L83】
- The recursive expression walker enforces capability usage, return correctness,
  and effect annotations through helper methods (not fully shown here), providing
  scaffolding for the richer static analysis milestones in the roadmap.【F:src/semantics/check.rs†L164-L400】【F:docs/roadmap/compiler.md†L97-L170】
- Diagnostics are aggregated into `CheckResult` so the CLI and tests can surface
  warnings without panicking, aligning with the roadmap’s quality gates.【F:src/semantics/check.rs†L5-L21】

## Integration Highlights

1. Resolution results feed the checker with symbol metadata and capability
   context, ensuring consistent treatment of imports and ADT variants.
2. CLI `--resolve` and `--check` modes expose semantic information for manual and
   automated validation, which is critical for the roadmap’s IDE and tooling
   deliverables.【F:src/main.rs†L64-L175】【F:docs/roadmap/tooling.md†L1-L60】
3. Lowering and IR generation consume the resolved module data to attach stable
   identifiers in later phases.【F:src/lower/mod.rs†L1-L200】

## Roadmap Alignment

- **Phase 1:** Current passes already implement early versions of name resolution
  and effect checking outlined for this phase, with room to expand into full type
  inference and borrow checking.【F:docs/roadmap/compiler.md†L76-L170】
- **Phase 2:** The structured `Resolved` output and signature collection will feed
  SSA lowering and optimization passes once those milestones begin.【F:docs/roadmap/compiler.md†L126-L170】
- **Tooling Roadmap:** The resolver’s rich metadata supports symbol search,
  documentation generation, and LSP indexers slated for the tooling milestones.【F:docs/roadmap/tooling.md†L1-L60】

## Next Steps

- Extend capability checks with lifetimes and region inference as described in
  the roadmap’s borrow-checking plans.
- Emit structured diagnostics from both resolver and checker to integrate with
  the planned language server protocol implementation.
- Add caching and incremental recomputation hooks to handle larger workspaces.
