# Project Status — Phase 3 Kickoff

_Last reviewed: 2024-06-25_

The current milestone focuses on stabilising the Phase 3 runtime and backend
work. The sections below summarise what is healthy today, the verification that
backs it up, and the immediate priorities for the next iteration.

## Current Health Check

- **Compiler pipeline** – The lexer, parser, resolver, and effect checker remain
  feature-complete for the prototype. Cross-stage integration tests exercise the
  entire front-end to keep behaviour aligned with the design tour.
- **Typed IR** – Lowering produces canonical types, reuses arena-backed
  metadata, and annotates purity so future optimisation passes have the data
  they need without duplicating computation.
- **Backends** – Textual and LLVM renderers are stable, while the native backend
  emits portable C that links with the system toolchain and respects capability
  contracts.
- **Diagnostics** – Capability misuse, duplicate effects, and missing bindings
  produce focused errors that ship with regression coverage to prevent
  accidental regressions.
- **Runtime** – Generated binaries thread capability providers through the
  runtime shim, enforcing declared effects before IO or time operations run.

## Verification Snapshot

- The CI workflow builds with locked dependencies, runs the entire test suite,
  regenerates documentation snippets, and performs a native smoke test.
- Fifty-plus integration suites cover lexing, parsing, resolving, lowering, IR
  emission, backend rendering, and execution paths.
- Documentation snippets are derived from real compiler output, ensuring the
  tour and CLI docs stay in sync with the codebase.

## Near-Term Priorities

1. Expand the capability provider library (filesystem, networking, process
   orchestration) so runtime-aware binaries cover more scenarios out of the box.
2. Emit structured runtime telemetry that feeds the forthcoming observability
   and tooling workstreams.
3. Exercise the parallel backend harness against multi-module workspaces to
   profile contention and tune scheduling heuristics.
4. Build CLI conveniences for the resolver and IR JSON dumps so editors and
   automation can consume the data without custom parsers.

## Risks & Watch Items

- **Provider coverage** – Console/time shims ship today; widening the surface
  without additional regression coverage risks unexpected behaviour.
- **Coverage portability** – Test coverage reporting remains paused until a
  cross-platform workflow is identified that does not require bespoke tooling.
- **Testing debt** – Parser and resolver fuzzing is still outstanding and should
  be revisited once backend work stabilises.

## Next Update

Revisit this status report after runtime wiring and structured CLI outputs
advance, or once the parallel backend experiments complete.
