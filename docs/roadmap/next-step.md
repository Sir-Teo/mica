# Immediate Next Step Recommendation

After the recent semantics push, the project now satisfies the Phase 1
milestones: the resolver builds a cross-module workspace, reports duplicate
symbols, and records capability metadata for downstream consumers, while the new
type/effect checker validates function signatures, capability usage, and match
exhaustiveness through a single CLI entry point. `mica --check` now drives the
enhanced checker so mismatches surface with actionable diagnostics. 【F:src/semantics/resolve/collector.rs†L6-L145】【F:src/semantics/resolve/resolver.rs†L9-L406】【F:src/semantics/check.rs†L1-L604】

With Phase 1 complete, the focus shifts to the next tranche of roadmap tasks:

1. **Design the typed IR for lowering (Phase 2).** Document the SSA-inspired IR
   layout, encode effect metadata, and begin lowering typed ASTs into this IR so
   later passes can reason about drops and scheduling. 【F:docs/roadmap/compiler.md†L120-L191】
2. **Seed backend integration work.** Start sketching the LLVM-oriented backend
   scaffolding that will eventually consume the IR, ensuring the lowering data
   model keeps enough provenance for code generation. 【F:docs/roadmap/compiler.md†L193-L245】
3. **Grow the diagnostics playbook.** Capture the semantics of the new
   type/effect errors inside the roadmap so LSP and tooling efforts can build on
   consistent wording and spans, and expand the CLI examples to demonstrate the
   richer checking pipeline. 【F:docs/roadmap/compiler.md†L60-L118】

Clearing these items transitions the project into Phase 2, setting the stage for
SSA lowering, backend work, and the optimizer research called out in the
compiler roadmap.
