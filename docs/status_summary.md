# Phase 2 Delivery Status

_Last refreshed: 2025-09-30 00:00 UTC_

## Where We Stand Today

### Front-end and Semantic Analysis
- Lexer, parser, and AST builders continue to live in `src/syntax`, giving us the full front-end pipeline expected for the Phase 1 baseline.【F:src/syntax/mod.rs†L1-L4】
- Resolver and capability tracking wire modules into workspace graphs, while the semantic checker enforces signatures and exhaustiveness across pattern matches.【F:src/semantics/resolve/mod.rs†L1-L61】【F:src/semantics/check.rs†L1-L200】
- Parser and semantic regression suites exercise ADTs, control-flow constructs, capability rows, and diagnostics so we keep catching integration bugs at the language level.【F:src/tests/parser_tests.rs†L4-L159】【F:src/tests/resolve_and_check_tests.rs†L120-L162】

### Lowering, Typed IR, and Backends
- The lowering layer converts ASTs into the homogeneous HIR (`HModule`, `HFunction`, `HExpr`), normalizing method calls, capability rows, and structured control flow before typed IR generation.【F:src/lower/mod.rs†L1-L198】
- Typed SSA IR modules track interned types/effects and today support literal, record, call, and arithmetic instructions plus return terminators, matching the current roadmap baseline.【F:src/ir/mod.rs†L1-L200】
- Lowering and IR tests assert coverage for method desugaring, structured expressions, effect rows, and return behaviour so we know the pipeline handles the language surface we ship today.【F:src/tests/lowering_tests.rs†L4-L191】【F:src/tests/ir_tests.rs†L4-L168】
- Text and LLVM scaffolding backends now render branches, phi nodes, string interning, and effect annotations so engineers can validate control flow and metadata without a native toolchain.【F:src/backend/text.rs†L1-L134】【F:src/backend/llvm.rs†L1-L320】【F:src/tests/backend_tests.rs†L100-L223】

### Tooling and Developer Experience
- The `mica` CLI routes a single parsed module through lexing, pretty-printing, checking, resolving, lowering, IR dumps, and the LLVM preview, which keeps day-to-day validation quick while we iterate on backends.【F:src/main.rs†L17-L215】
- Documentation for the CLI and snippet generator mirrors the exposed modes so contributors understand how to reproduce backend snapshots and diagnostics locally.【F:docs/modules/cli.md†L12-L60】

## Verification Snapshot
- The test harness covers lexer, parser, lowering, IR, backend, resolver, and diagnostics suites (37 unit-style tests today), confirming every stage of the pipeline with golden expectations and negative cases.【F:src/tests/mod.rs†L1-L17】【F:src/tests/pipeline_tests.rs†L5-L139】
- Backend and pipeline tests keep the effect system, capability metadata, and match diagnostics stable while we refine IR semantics.【F:src/tests/backend_tests.rs†L4-L223】【F:src/tests/resolve_and_check_tests.rs†L120-L162】

## Gap Analysis Against Phase 2 Exit Criteria
- LLVM record lowering and named aggregates still route through a placeholder stub and collapse to opaque pointers, so we need a real struct layout story and metadata plumbing before Phase 3 backend work can begin.【F:src/backend/llvm.rs†L256-L320】【F:src/tests/backend_tests.rs†L198-L223】【F:docs/roadmap/compiler.md†L170-L215】
- The LLVM backend continues to emit textual scaffolding without instruction selection or data layout integration, so native code generation and runtime interop remain future tasks.【F:src/backend/llvm.rs†L1-L320】【F:docs/roadmap/compiler.md†L137-L156】
- Diagnostics largely cover exhaustiveness and signature checks; capability misuse, borrow-like flows, and backend-specific failures still need regression fixtures to match the roadmap goals.【F:src/semantics/check.rs†L15-L200】【F:docs/roadmap/milestones.md†L37-L45】
- Purity/effect analysis has not started, leaving the structured concurrency research item untouched for Phase 2.【F:docs/roadmap/milestones.md†L37-L45】

## Recommended Next Actions
1. **Model aggregates faithfully**: Teach lowering and the LLVM backend to materialize record layouts (fields, padding, provenance) instead of the current pointer stub so named types carry real structure into downstream passes.【F:src/lower/mod.rs†L1-L220】【F:src/backend/llvm.rs†L256-L320】【F:docs/roadmap/compiler.md†L170-L215】
2. **Move beyond scaffolding**: Introduce basic instruction selection and data layout awareness in the LLVM backend so arithmetic, calls, and effects compile into runnable modules rather than textual placeholders.【F:src/backend/llvm.rs†L1-L320】【F:src/tests/backend_tests.rs†L100-L223】【F:docs/roadmap/compiler.md†L137-L192】
3. **Grow diagnostics coverage**: Introduce targeted negative tests for capability misuse, missing effect declarations, and backend contract violations so Phase 2 exit checks stay green during IR evolution.【F:src/tests/resolve_and_check_tests.rs†L120-L162】【F:docs/roadmap/milestones.md†L37-L45】
4. **Prototype purity analysis**: Capture effect metadata in lowering and sketch analyses that identify effect-free regions, storing results alongside the IR for future auto-parallelization experiments.【F:src/lower/mod.rs†L68-L198】【F:docs/roadmap/milestones.md†L37-L45】

## Watch Items
- Shared type/effect tables in the IR will need concurrency-safe access once multiple backends consume a module; design the ownership story before parallel compilation enters the picture.【F:src/ir/mod.rs†L97-L147】
- CLI outputs are textual today; consider structured formats so tooling built during Phase 3+ can ingest resolver and IR data without fragile scraping.【F:src/main.rs†L51-L201】【F:docs/modules/cli.md†L54-L60】
