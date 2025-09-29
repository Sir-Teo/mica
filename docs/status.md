# Project Status — Phase 2 Kickoff

_Last updated: 2025-09-29 23:23 UTC_

## Current Health Check
- **Compiler pipeline**: Lexer, parser, resolver, and type checker are implemented and covered by regression tests.
- **Typed IR**: Lowerer assigns stable `TypeId`s with shared registries; CLI exposes `mica --ir` for inspection.
- **Backend**: Text emitter backend renders typed IR for validation and future backend adapters.
- **Diagnostics**: Playbook refreshed with Phase 2 criteria; CLI documentation mirrors current surface area.

## Test & Verification Snapshot
- `cargo test` (unit + integration) — all 36 suites pass locally.
- Doc tests for CLI utilities execute with zero regressions.

## Near-Term Priorities
1. Flesh out typed IR coverage for control-flow joins, pattern destructuring, and effect polymorphism.
2. Design backend trait abstractions that can target LLVM and WASM while reusing registries.
3. Extend diagnostics regression matrix with IR-specific failure fixtures.
4. Explore purity analysis for structured concurrency primitives ahead of runtime work.

## Risks & Watch Items
- **Registry Sharing**: Need to validate concurrency story when multiple backends request metadata.
- **CLI UX**: `mica --ir` currently emits text only; consider JSON/protobuf for tooling.
- **Testing Debt**: No fuzzing yet for parser/resolver; schedule once Phase 2 IR features stabilize.

## Next Status Update
- Revisit after backend trait RFC merges or when IR lowering milestones shift.
