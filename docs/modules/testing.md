# Testing Harness

> The regression suite mirrors the entire compiler pipeline so feature work has
> fast, reliable feedback.

## Overview

The `src/tests` directory houses unit and integration suites that exercise every
compiler stage. Lexing, parsing, resolving, lowering, IR generation, backend
rendering, and runtime execution all have dedicated coverage. Keeping these
suites green is the fastest way to confirm the language tour still matches the
code.

## Structure

- **Foundational suites** – Lexer, parser, and pretty-printer tests assert the
  surface syntax and formatting contracts.
- **Semantic suites** – Resolver and checker coverage verifies imports,
  capability tracking, effect rows, and diagnostics.
- **Pipeline suites** – Lowering, IR, backend, and pipeline tests catch
  regressions that span multiple stages, including native execution snapshots.
- **Shared fixtures** – `helpers.rs` loads examples and normalises diagnostics so
  suites can focus on behaviour rather than setup.

## How to Use the Harness

1. Run `cargo test --all-targets` after landing feature work to see the full
   pipeline results.
2. Use the snapshot-backed tests (`display`, `pretty`, `pipeline`) as inspiration
   when documenting new CLI flows or examples.
3. Add targeted fixtures whenever a bug reproducer is available; the suites are
   intentionally lightweight so contributors can drop in new cases quickly.

## Alignment with the Roadmap

- **Phase 0–1** – Ensures the front-end stays stable as syntax and semantics
  expand.
- **Phase 2** – Guards lowering and SSA output as optimisation groundwork is
  laid.
- **Tooling** – Feeds accurate output into the documentation snippets and future
  IDE integrations.

## Next Steps

- Expand integration coverage to multi-module workspaces once resolver
  enhancements land.
- Explore property-based tests for lowering and SSA transformations to catch
  subtle corner cases.
- Re-introduce coverage reporting to visualise progress toward roadmap goals.
