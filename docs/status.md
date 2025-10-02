# Project Status — Phase 3 Kickoff

_Last updated: 2025-10-05 00:00 UTC_

## Current Health Check
- **Compiler pipeline**: Lexer, parser, resolver, and type checker remain healthy with broad regression coverage across the front-end suites.【F:src/semantics/check.rs†L1-L960】【F:src/tests/parser_tests.rs†L4-L159】
- **Typed IR**: Lowering interns canonical types/effects, records concrete aggregate layouts, ships purity analysis, and now shares metadata through copy-on-write arenas so multiple backends can reuse registries safely.【F:src/ir/mod.rs†L1-L215】【F:src/ir/mod.rs†L780-L940】【F:src/ir/analysis.rs†L1-L140】
- **Backend**: Text and LLVM renderers continue to enforce layout and effect contracts, while the new native backend compiles typed IR to portable C and links executables through the system toolchain.【F:src/backend/text.rs†L1-L134】【F:src/backend/llvm.rs†L1-L420】【F:src/backend/native.rs†L1-L400】
- **Diagnostics**: Capability misuse, duplicate effects, and missing bindings surface dedicated errors with regression coverage in the test suite.【F:src/semantics/check.rs†L650-L940】【F:src/tests/resolve_and_check_tests.rs†L1-L210】

## Test & Verification Snapshot
- The CI pipeline now enforces formatting and linting, compiles documentation, runs the full test matrix, executes the snippet check, performs a native backend smoke test against `examples/native_entry.mica`, and gates coverage at 50% line execution via `cargo llvm-cov`.【F:.github/workflows/ci.yml†L1-L77】【F:examples/native_entry.mica†L1-L10】
- `cargo test` (unit + integration) — 55 suites cover lexer, parser, lowering, IR, backend, resolver, diagnostics, and the native execution path.【F:src/tests/mod.rs†L1-L17】【F:src/tests/backend_tests.rs†L320-L382】

## Near-Term Priorities
1. Stand up capability-aware runtime shims and scheduling hooks that honour effect metadata, unblocking IO/task experiments in Phase 3.【F:docs/roadmap/compiler.md†L200-L240】
2. Harden native execution diagnostics so link or toolchain failures surface structured messages instead of raw exit codes.【F:src/backend/native.rs†L141-L199】【F:src/main.rs†L210-L260】
3. Prototype parallel backend execution using the shared registries to validate contention and concurrency guarantees.【F:src/ir/mod.rs†L100-L215】【F:src/ir/mod.rs†L780-L940】
4. Raise the CI coverage gate as runtime features land and extend diagnostics toward borrow flows and backend validation while maintaining the richer regression suite established in Phase 2.【F:.github/workflows/ci.yml†L39-L71】【F:docs/roadmap/milestones.md†L37-L60】

## Risks & Watch Items
- **Native backend coverage**: The portable C emitter currently omits record/aggregate lowering; expand support before compiling complex data-heavy examples.【F:src/backend/native.rs†L230-L320】
- **CLI UX**: Compiler outputs remain textual; structured formats will be required for tooling consumers as Phase 3 progresses.【F:src/main.rs†L63-L205】【F:docs/modules/cli.md†L60-L80】
- **Testing debt**: Parser/resolver fuzzing is still on the backlog; re-evaluate once codegen stabilizes post-Phase 3 kickoff.

## Next Status Update
- Revisit after runtime capability shims land or once the native backend covers aggregate lowering and richer diagnostics.
