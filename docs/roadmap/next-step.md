# Immediate Next Step Recommendation

After reviewing the existing documentation and source layout, the project has
solid coverage of the Phase 0 goals: the lexer, parser, pretty printer, and CLI
(`mica --tokens`, `--ast`, `--pretty`, `--resolve`, `--lower`) are implemented
with snapshot-style tests. The resolver was recently expanded to collect
declarations, maintain scoped symbol tables, capture imports, and record
capability requirements, giving us the groundwork needed for the rest of Phase 1
semantics. 【F:src/semantics/resolve.rs†L1-L390】

The remaining semantic layer still has notable gaps:

* `resolve::resolve_module` does not yet validate unresolved paths, surface
  diagnostics for duplicate bindings, or load cross-module dependencies.
  Extending it toward multi-module analysis will be necessary for the compiler
  CLI and LSP to scale. 【F:src/semantics/resolve.rs†L392-L701】
* `check::check_exhaustiveness` is limited to match exhaustiveness warnings and
  does not yet perform Hindley–Milner inference, borrow checking, or
  capability/effect validation. 【F:src/semantics/check.rs†L1-L92】

**Recommendation**: Begin the type and effect checker implementation. Start by
consuming the resolver’s symbol tables to build an environment for function and
type declarations, add capability validation hooks, and incrementally introduce
type inference for function bodies. As that machinery grows, fold in
cross-module resolution and diagnostics so the resolver can flag missing imports
and duplicate definitions ahead of type checking. This lines up with the Phase 1
plan in the compiler roadmap and positions the project for richer diagnostics
and IDE integrations. 【F:docs/roadmap/compiler.md†L41-L130】
