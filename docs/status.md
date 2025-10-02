# Project Status — Phase 3 Kickoff

_Last updated: 2025-10-06 00:00 UTC_

## Current Health Check
- **Compiler pipeline**: Lexer, parser, resolver, and type checker remain healthy with broad regression coverage across the front-end suites.【F:src/semantics/check.rs†L1-L960】【F:src/tests/parser_tests.rs†L4-L159】
- **Typed IR**: Lowering interns canonical types/effects, records concrete aggregate layouts, ships purity analysis, and now shares metadata through copy-on-write arenas so multiple backends can reuse registries safely.【F:src/ir/mod.rs†L1-L215】【F:src/ir/mod.rs†L780-L940】【F:src/ir/analysis.rs†L1-L140】
- **Backend**: Text and LLVM renderers continue to enforce layout and effect contracts, while the native backend now lowers records alongside scalars, emitting portable C that links through the system toolchain.【F:src/backend/text.rs†L1-L134】【F:src/backend/llvm.rs†L1-L420】【F:src/backend/native.rs†L1-L640】
- **Diagnostics**: Capability misuse, duplicate effects, and missing bindings surface dedicated errors with regression coverage in the test suite.【F:src/semantics/check.rs†L650-L940】【F:src/tests/resolve_and_check_tests.rs†L1-L210】
- **Runtime**: The native backend now threads capability providers directly into generated executables; runtime guards validate declared effects and route operations through the built-in IO/time shims before execution continues.【F:src/backend/native.rs†L1-L720】【F:src/runtime/mod.rs†L1-L520】

## Test & Verification Snapshot
- The CI pipeline enforces formatting and linting, compiles documentation, runs the full test matrix, executes the snippet check, and performs a native backend smoke test; coverage reporting remains paused until we reinstate a portable workflow.【F:.github/workflows/ci.yml†L1-L53】【F:examples/native_entry.mica†L1-L10】
- `cargo test` (unit + integration) — 55 suites cover lexer, parser, lowering, IR, backend, resolver, diagnostics, and the native execution path.【F:src/tests/mod.rs†L1-L17】【F:src/tests/backend_tests.rs†L320-L382】

## Near-Term Priorities
1. Expand the provider library (filesystem, networking, process orchestration) so runtime-aware binaries cover more tour scenarios out of the box.【F:src/backend/native.rs†L1-L720】【F:src/runtime/mod.rs†L1-L520】
2. Emit structured runtime telemetry from executables to feed forthcoming tooling and observability workstreams.【F:src/runtime/mod.rs†L260-L480】【F:docs/modules/tooling.md†L20-L60】
3. Exercise the parallel backend harness against multi-module workspaces to profile contention and tune scheduling heuristics.【F:src/backend/mod.rs†L1-L200】【F:src/ir/mod.rs†L1-L1120】
4. Build CLI conveniences over the resolver/IR JSON dumps so editors and pipeline tooling can consume the data without custom parsing.【F:src/main.rs†L20-L360】【F:docs/modules/cli.md†L60-L80】

## Risks & Watch Items
- **Provider coverage**: Only console/time shims are bundled; widen integration tests before shipping additional host capabilities to avoid surprising consumers.【F:src/backend/native.rs†L1-L720】【F:src/tests/backend_tests.rs†L320-L420】
- **Coverage portability**: Track options for reintroducing coverage that run reliably across Linux, macOS, and Windows runners without bespoke tool installs.【F:.github/workflows/ci.yml†L1-L53】
- **Testing debt**: Parser/resolver fuzzing is still on the backlog; re-evaluate once codegen stabilizes post-Phase 3 kickoff.

## Next Status Update
- Revisit after runtime wiring and structured CLI outputs advance, or once parallel backend experiments shake out concurrency risks.
