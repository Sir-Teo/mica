# Phase 3 Kickoff Status

_Last refreshed: 2024-06-25_

This summary condenses the current Phase 3 health report for quick scanning. It
highlights what is stable, how we validate it, and where contributors can have
the most impact next.

## Snapshot of the Pipeline

### Front-end and Semantic Analysis
- Lexer, parser, and AST builders provide the full front-end baseline promised
  for the first milestones. Behaviour stays aligned with the language tour
  through targeted regression suites.
- Resolver and capability tracking build workspace graphs and enforce signatures
  and effect rows so capability misuse is caught at compile time.
- Parser and semantic tests cover capability misuse, negative effect rows,
  algebraic data types, control-flow constructs, and diagnostics.

### Lowering, Typed IR, and Backends
- Lowering converts ASTs into a homogeneous HIR, normalising method calls,
  capability rows, and structured control flow before IR generation.
- Typed SSA modules reuse shared registries which keeps metadata cheap to clone
  and opens the door to parallel backends.
- Lowering and IR tests assert coverage for method desugaring, structured
  expressions, effect rows, return behaviour, and purity reporting.
- The native backend emits portable C for typed IR, including record aggregates,
  and drives the system toolchain to produce runnable binaries.
- The runtime orchestrator binds IO/time providers and validates capability
  coverage before native binaries execute.

### Tooling and Developer Experience
- The `mica` CLI exposes lexing, resolving, lowering, IR dumps, LLVM previews,
  and native build/run flows so contributors can exercise every compiler stage.
- Documentation for the CLI and snippet generator mirrors the exposed modes,
  helping new contributors reproduce backend snapshots and diagnostics locally.

## Verification Snapshot

- CI enforces formatting, linting, documentation builds, test execution, and
  snippet verification. A native backend smoke test ensures generated binaries
  still link and execute.
- The test harness covers lexer, parser, lowering, IR, backend, resolver, and
  diagnostic suites (55 unit-style tests today) with golden expectations and
  negative cases.
- Backend and pipeline tests keep the effect system, capability metadata, and
  match diagnostics stable while IR semantics evolve.

## Phase 2 Exit Criteria (All Met)

- **Aggregate modelling** – Record layouts now carry concrete offsets, size, and
  alignment metadata consumed directly by the LLVM backend.
- **Backend fidelity** – The LLVM emitter annotates blocks with purity details,
  enforces record layout availability, and surfaces unsupported constructs.
- **Diagnostics depth** – Semantic checks raise duplicate capability, missing
  binding, and scope violation errors with regression coverage.
- **Purity analysis** – SSA functions expose effect-free regions that pave the
  way for future parallelisation work.

## Next Focus Areas

1. **Provider breadth** – Extend runtime shims beyond console/time to cover
   filesystem and networking scenarios.
2. **Runtime telemetry** – Emit structured execution events from generated
   binaries for downstream observability tooling.
3. **Parallel backend scaling** – Stress the parallel compile driver across
   workspace-sized module sets and instrument contention hotspots.
4. **Tooling APIs** – Layer higher-level CLI entry points over the resolver and
   IR JSON dumps to unblock IDE integrations and automated audits.

## Watch Items

- Runtime provider coverage is intentionally narrow; broaden it only with new
  smoke tests to avoid surprising consumers.
- Evaluate portable coverage collectors that work across the CI matrix so we can
  reintroduce reporting without bespoke tooling.
