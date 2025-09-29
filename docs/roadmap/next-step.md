# Immediate Next Step Recommendation

After reviewing the existing documentation and source layout, the project has
solid coverage of the Phase 0 goals: the lexer, parser, pretty printer, and CLI
(`mica --tokens`, `--ast`, `--pretty`, `--resolve`, `--lower`) are implemented
with snapshot-style tests. The resolver is now split into a declaration
collection pass and a scoped resolver. The collector seeds module-level symbol
tables, imports, and algebraic data type metadata, while the resolver walks the
AST to bind paths and capability annotations into those tables. This gives us
the groundwork needed for the rest of the Phase 1 semantics. 【F:src/semantics/resolve/collector.rs†L6-L114】【F:src/semantics/resolve/resolver.rs†L9-L205】

The remaining semantic layer still has notable gaps:

* `resolve_module` only gathers local symbols before resolving in-module
  references. Unresolved paths simply record `None`, and there is currently no
  surface for duplicate-definition diagnostics or loading the targets of
  `use` items from other files. Extending this toward cross-module analysis will
  be necessary for the compiler CLI and future LSP support to scale. 【F:src/semantics/resolve/mod.rs†L1-L21】【F:src/semantics/resolve/resolver.rs†L206-L314】
* `check::check_exhaustiveness` is limited to match exhaustiveness warnings and
  does not yet perform Hindley–Milner inference, borrow checking, or
  capability/effect validation. 【F:src/semantics/check.rs†L1-L147】

## Recommended next steps

1. **Harden the resolver for multi-module work.**
   * Extend the collector to persist per-module import metadata in a form that
     other files can consume, building a module graph that feeds follow-up
     resolution passes. 【F:src/semantics/resolve/collector.rs†L6-L114】
   * Teach `Resolver::resolve_path` to emit diagnostics when bindings are
     missing or duplicated once cross-module lookups are wired in; capture spans
     so the CLI and future LSP consumers can highlight errors precisely.
     【F:src/semantics/resolve/resolver.rs†L31-L205】【F:src/semantics/resolve/resolver.rs†L206-L314】
   * Add regression tests (single-file and synthetic multi-module fixtures) that
     exercise shadowing, duplicate definitions, and `use` re-exports before
     evolving the type checker.

2. **Bootstrap the type and effect checker.**
   * Define type environments sourced from the resolver’s symbol tables and
     begin unification plumbing for expressions, producing placeholder type IDs
     that later lowering stages can consume. 【F:src/semantics/check.rs†L1-L147】
   * Add capability validation hooks that reuse the resolver’s capability
     bindings so exhaustiveness and capability diagnostics flow through a single
     entry point.
   * Iterate with golden tests driven via `mica --check` to ensure early
     regressions are caught and to document the desired diagnostic UX.

3. **Align roadmap artifacts and tooling.**
   * Update the compiler roadmap once the resolver/type-checker milestones above
     land so the broader plan reflects real module boundaries and exit criteria.
     【F:docs/roadmap/compiler.md†L60-L110】
   * Promote the new resolver and checker stages through the CLI by wiring
     `--resolve` and `--check` outputs into example fixtures, ensuring people
     experimenting with the language see up-to-date capabilities.

These steps keep progress focused on Phase 1 priorities while clearing the most
visible gaps that block richer diagnostics and editor integrations. They also
feed directly into the roadmap’s planned type-and-effect workstreams.
