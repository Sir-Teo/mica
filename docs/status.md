# Project Status — Phase 3 Kickoff

_Last updated: 2025-10-06 00:00 UTC_

## Current Health Check
- **Compiler pipeline**: Lexer, parser, resolver, and type checker remain healthy with broad regression coverage across the front-end suites.【F:src/semantics/check.rs†L1-L960】【F:src/tests/parser_tests.rs†L4-L159】
- **Typed IR**: Lowering interns canonical types/effects, records concrete aggregate layouts, ships purity analysis, and now shares metadata through copy-on-write arenas so multiple backends can reuse registries safely.【F:src/ir/mod.rs†L1-L215】【F:src/ir/mod.rs†L780-L940】【F:src/ir/analysis.rs†L1-L140】
- **Backend**: Text and LLVM renderers continue to enforce layout and effect contracts, while the native backend now lowers records alongside scalars, emitting portable C that links through the system toolchain.【F:src/backend/text.rs†L1-L134】【F:src/backend/llvm.rs†L1-L420】【F:src/backend/native.rs†L1-L640】
- **Diagnostics**: Capability misuse, duplicate effects, and missing bindings surface dedicated errors with regression coverage in the test suite.【F:src/semantics/check.rs†L650-L940】【F:src/tests/resolve_and_check_tests.rs†L1-L210】
- **Runtime**: A capability-aware runtime orchestrator binds IO/time shims, enforces deterministic FIFO scheduling, and now backs the CLI `--run` path by validating entrypoint capability requirements before executing binaries.【F:src/runtime/mod.rs†L1-L380】【F:src/main.rs†L210-L320】

## Test & Verification Snapshot
- The CI pipeline now enforces formatting and linting, compiles documentation, runs the full test matrix, executes the snippet check, and performs a native backend smoke test against `examples/native_entry.mica`; coverage collection via `cargo llvm-cov` is temporarily paused while we rework the job for restricted environments.【F:.github/workflows/ci.yml†L1-L55】【F:examples/native_entry.mica†L1-L10】
- `cargo test` (unit + integration) — 55 suites cover lexer, parser, lowering, IR, backend, resolver, diagnostics, and the native execution path.【F:src/tests/mod.rs†L1-L17】【F:src/tests/backend_tests.rs†L320-L382】

## Near-Term Priorities
1. Thread runtime events and capability handles into the generated binaries so compiled programs can invoke providers directly instead of only validating coverage pre-run.【F:src/backend/native.rs†L200-L420】【F:src/main.rs†L210-L320】
2. Extend native execution diagnostics so unsupported IR constructs surface actionable errors alongside the richer link failure messages.【F:src/backend/native.rs†L200-L360】【F:src/main.rs†L210-L320】
3. Prototype parallel backend execution using the shared registries to validate contention and concurrency guarantees.【F:src/ir/mod.rs†L100-L215】【F:src/ir/mod.rs†L780-L940】
4. Reinstate a portable coverage job (targeting `cargo llvm-cov` or an equivalent) once the environment constraints are resolved, then continue raising the gate alongside richer diagnostics and backend validation as runtime features land.【F:.github/workflows/ci.yml†L1-L55】【F:docs/roadmap/milestones.md†L37-L60】

## Risks & Watch Items
- **Runtime integration**: The CLI now guards capability coverage ahead of execution, but compiled binaries still bypass provider shims at runtime; wire the generated code to invoke capability handlers before expanding IO-heavy examples.【F:src/runtime/mod.rs†L1-L380】【F:src/main.rs†L210-L320】
- **CLI UX**: Compiler outputs remain textual; structured formats will be required for tooling consumers as Phase 3 progresses.【F:src/main.rs†L63-L205】【F:docs/modules/cli.md†L60-L80】
- **Testing debt**: Parser/resolver fuzzing is still on the backlog; re-evaluate once codegen stabilizes post-Phase 3 kickoff.

## Next Status Update
- Revisit after runtime wiring and structured CLI outputs advance, or once parallel backend experiments shake out concurrency risks.
