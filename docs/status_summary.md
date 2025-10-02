# Phase 3 Kickoff Status

_Last refreshed: 2025-10-06 00:00 UTC_

## Where We Stand Today

### Front-end and Semantic Analysis
- Lexer, parser, and AST builders continue to live in `src/syntax`, giving us the full front-end pipeline expected for the Phase 1 baseline.【F:src/syntax/mod.rs†L1-L4】
- Resolver and capability tracking wire modules into workspace graphs, while the semantic checker enforces signatures, effect rows, and exhaustiveness across pattern matches.【F:src/semantics/resolve/mod.rs†L1-L61】【F:src/semantics/check.rs†L1-L960】
- Parser and semantic regression suites now cover capability misuse, negative effect rows, ADTs, control-flow constructs, and diagnostics so we keep catching integration bugs at the language level.【F:src/tests/parser_tests.rs†L4-L159】【F:src/tests/resolve_and_check_tests.rs†L1-L210】

### Lowering, Typed IR, and Backends
- The lowering layer converts ASTs into the homogeneous HIR (`HModule`, `HFunction`, `HExpr`), normalizing method calls, capability rows, and structured control flow before typed IR generation.【F:src/lower/mod.rs†L1-L640】
- Typed SSA IR modules now share their type/effect registries through copy-on-write arenas so multiple backends can consume a module without cloning metadata, paving the way for parallel compilation.【F:src/ir/mod.rs†L100-L215】【F:src/ir/mod.rs†L780-L940】
- Lowering and IR tests assert coverage for method desugaring, structured expressions, effect rows, return behaviour, and purity reporting so we know the pipeline handles the language surface we ship today.【F:src/tests/lowering_tests.rs†L1-L200】【F:src/tests/ir_tests.rs†L1-L360】
- The native backend now emits portable C for typed IR, including record aggregates, drives the system toolchain to produce binaries, and is covered by executable regression tests alongside the text and LLVM preview emitters.【F:src/backend/native.rs†L1-L640】【F:src/tests/backend_tests.rs†L300-L420】
- A capability-aware runtime orchestrator binds IO/time providers, enforces deterministic scheduling, and now backs the CLI `--run` path by validating entrypoint capability coverage before native binaries execute.【F:src/runtime/mod.rs†L1-L380】【F:src/main.rs†L210-L320】【F:src/tests/runtime_tests.rs†L1-L180】

### Tooling and Developer Experience
- The `mica` CLI now includes native `--build` and `--run` flows in addition to lexing, resolving, lowering, IR dumps, and the LLVM preview so contributors can execute examples end-to-end through the new backend, with runtime-backed capability validation prior to execution.【F:src/main.rs†L17-L320】
- Documentation for the CLI and snippet generator mirrors the exposed modes so contributors understand how to reproduce backend snapshots and diagnostics locally.【F:docs/modules/cli.md†L12-L80】

## Verification Snapshot
- CI now enforces formatting and clippy linting, builds docs, runs the full workspace test matrix, verifies CLI snippets, and smokes the native backend via `examples/native_entry.mica`; coverage reporting is paused until we land a portable replacement for the prior `cargo llvm-cov` job.【F:.github/workflows/ci.yml†L1-L55】【F:examples/native_entry.mica†L1-L10】
- The test harness covers lexer, parser, lowering, IR, backend, resolver, and diagnostics suites (55 unit-style tests today), confirming every stage of the pipeline with golden expectations and negative cases.【F:src/tests/mod.rs†L1-L17】【F:src/tests/backend_tests.rs†L320-L382】
- Backend and pipeline tests keep the effect system, capability metadata, and match diagnostics stable while we refine IR semantics.【F:src/tests/backend_tests.rs†L1-L382】【F:src/tests/resolve_and_check_tests.rs†L1-L210】

## Phase 2 Exit Criteria
- ✅ **Aggregate modelling**: Record layouts now carry concrete offsets, size, and alignment metadata that the LLVM backend renders directly, eliminating the placeholder struct stub from earlier milestones.【F:src/ir/mod.rs†L700-L940】【F:src/backend/llvm.rs†L180-L340】【F:src/tests/backend_tests.rs†L200-L260】
- ✅ **Backend fidelity**: The LLVM emitter annotates blocks with purity information, enforces record layout availability, and surfaces errors for unsupported constructs so textual output mirrors the real codegen contract.【F:src/backend/llvm.rs†L1-L420】【F:src/tests/backend_tests.rs†L1-L308】
- ✅ **Diagnostics depth**: Semantic checks now surface duplicate capabilities, missing bindings, and scope violations with regression coverage in the test suite.【F:src/semantics/check.rs†L650-L940】【F:src/tests/resolve_and_check_tests.rs†L1-L210】
- ✅ **Purity analysis**: SSA functions include connectivity-aware purity reports that identify effect-free regions for future parallelization work.【F:src/ir/analysis.rs†L1-L140】【F:src/tests/ir_tests.rs†L280-L360】

## Next Focus Areas
1. **Runtime integration**: Wire generated binaries to invoke capability providers directly so runtime validation becomes full effect handling during execution.【F:src/backend/native.rs†L200-L420】【F:src/main.rs†L210-L320】
2. **Execution diagnostics**: Surface structured errors from the native pipeline (link failures, unsupported IR) instead of raw toolchain exits.【F:src/backend/native.rs†L200-L360】【F:src/main.rs†L210-L260】
3. **Parallel backend preparation**: Leverage the new copy-on-write registries to prototype parallel code generation and measure contention across threads.【F:src/ir/mod.rs†L100-L215】【F:src/ir/mod.rs†L780-L940】
4. **Structured CLI outputs**: Extend resolver/IR dumps with machine-readable formats to support upcoming tooling milestones.【F:src/main.rs†L63-L205】【F:docs/modules/cli.md†L60-L80】

## Watch Items
- Runtime validation precedes execution, but compiled executables still bypass provider shims at runtime; thread the handlers into generated code so capability metadata continues to matter end to end.【F:src/runtime/mod.rs†L1-L380】【F:src/main.rs†L210-L320】
- CLI outputs are textual today; consider structured formats so tooling built during Phase 3+ can ingest resolver and IR data without fragile scraping.【F:src/main.rs†L63-L205】【F:docs/modules/cli.md†L60-L80】
