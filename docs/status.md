# Project Status — Phase 3 Kickoff

_Last reviewed: 2025-10-04_

[← Back to Documentation Home](index.html)

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
  contracts. The parallel driver now records worker utilisation and scheduling
  timelines to guide optimisation work.
- **Diagnostics** – Capability misuse, duplicate effects, and missing bindings
  produce focused errors that ship with regression coverage to prevent
  accidental regressions.
- **Runtime** – Generated binaries thread capability providers through the
  runtime shim, enforcing declared effects before IO or time operations run.
  Deterministic, in-memory shims unblock regression tests without touching the
  host environment, and telemetry captures per-capability timings.

## Verification Snapshot

- The CI workflow builds with locked dependencies, runs the entire test suite,
  regenerates documentation snippets, and performs a native smoke test.
- Fifty-plus integration suites cover lexing, parsing, resolving, lowering, IR
  emission, backend rendering, and execution paths.
- Documentation snippets are derived from real compiler output, ensuring the
  tour and CLI docs stay in sync with the codebase.

## Near-Term Priorities

1. Introduce process and task-orchestration providers so sandboxed binaries can
   spawn scripted subprocesses without leaving the deterministic surface.
2. Feed the richer runtime telemetry into CLI exporters (e.g. `mica run`
   `--trace-json <path>`) and document downstream ingestion patterns.
3. Use the new parallel compile metrics to tune worker heuristics against
   workspace-sized module sets and surface the results in the docs.
4. Extend the freshly documented pipeline entry points with end-to-end
   walkthroughs that show how resolver and IR snapshots integrate with editor
   tooling.

## Risks & Watch Items

- **Provider coverage** – The deterministic shims reduce IO risk, but adding
  process orchestration requires careful sandboxing and regression coverage.
- **Coverage portability** – Test coverage reporting remains paused until a
  cross-platform workflow is identified that does not require bespoke tooling.
- **Testing debt** – Parser and resolver fuzzing is still outstanding and should
  be revisited once backend work stabilises.

## Next Update

Revisit this status report after runtime wiring and structured CLI outputs
advance, or once the parallel backend experiments complete.

---

## Related Documentation

- **[Status Summary](status_summary.html)** — Condensed Phase 3 health report
- **[Roadmap](roadmap/index.html)** — Full milestone breakdown
- **[Language Tour](tour.html)** — Explore the language features

[← Back to Documentation Home](index.html)
