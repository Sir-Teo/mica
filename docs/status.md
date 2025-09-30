# Project Status — Phase 2 Kickoff

_Last updated: 2025-09-29 23:23 UTC_

## Current Health Check
- **Compiler pipeline**: Lexer, parser, resolver, and type checker are implemented and covered by regression tests.
- **Typed IR**: Lowerer assigns stable `TypeId`s with shared registries; design notes now describe backend expectations and helpers like `Module::unknown_type` for consumers.【F:docs/modules/ir.md†L1-L70】
- **Backend**: Text emitter plus the new LLVM scaffolding backend render typed IR with effect metadata for validation and future native code generation.【F:src/backend/llvm.rs†L1-L226】【F:src/tests/backend_tests.rs†L1-L96】
- **Diagnostics**: Playbook refreshed with backend snapshots and CLI documentation mirrors the expanded surface area, including new IR examples.【F:docs/modules/diagnostics.md†L1-L89】【F:docs/snippets.md†L1-L80】

## Test & Verification Snapshot
- `cargo test` (unit + integration) — all 36 suites pass locally.
- Doc tests for CLI utilities execute with zero regressions.

## Near-Term Priorities
1. Flesh out typed IR coverage for control-flow joins, pattern destructuring, and effect polymorphism.
2. Harden the LLVM scaffolding into a codegen pipeline that lowers SSA blocks into real LLVM IR instructions.
3. Extend diagnostics regression matrix with IR-specific failure fixtures and backend error cases.
4. Explore purity analysis for structured concurrency primitives ahead of runtime work.

## Risks & Watch Items
- **Registry Sharing**: Need to validate concurrency story when multiple backends request metadata.
- **CLI UX**: `mica --ir` currently emits text only; consider JSON/protobuf for tooling.
- **Testing Debt**: No fuzzing yet for parser/resolver; schedule once Phase 2 IR features stabilize.

## Next Status Update
- Revisit after backend trait RFC merges or when IR lowering milestones shift.
