# Phase 2 Delivery Status

_Last refreshed: 2025-10-01 22:10 UTC_

## Where We Stand Today

### Front-end and Semantic Analysis
- Lexer, parser, and AST builders continue to live in `src/syntax`, giving us the full front-end pipeline expected for the Phase 1 baseline.【F:src/syntax/mod.rs†L1-L4】
- Resolver and capability tracking wire modules into workspace graphs, while the semantic checker enforces signatures, effect rows, and exhaustiveness across pattern matches.【F:src/semantics/resolve/mod.rs†L1-L61】【F:src/semantics/check.rs†L1-L960】
- Parser and semantic regression suites now cover capability misuse, negative effect rows, ADTs, control-flow constructs, and diagnostics so we keep catching integration bugs at the language level.【F:src/tests/parser_tests.rs†L4-L159】【F:src/tests/resolve_and_check_tests.rs†L1-L210】

### Lowering, Typed IR, and Backends
- The lowering layer converts ASTs into the homogeneous HIR (`HModule`, `HFunction`, `HExpr`), normalizing method calls, capability rows, and structured control flow before typed IR generation.【F:src/lower/mod.rs†L1-L640】
- Typed SSA IR modules track interned types/effects, compute concrete record layouts with padding metadata, and expose purity analysis that identifies effect-free regions for downstream backends.【F:src/ir/mod.rs†L1-L840】【F:src/ir/analysis.rs†L1-L140】
- Lowering and IR tests assert coverage for method desugaring, structured expressions, effect rows, return behaviour, and purity reporting so we know the pipeline handles the language surface we ship today.【F:src/tests/lowering_tests.rs†L1-L200】【F:src/tests/ir_tests.rs†L1-L360】
- Text and LLVM backends now emit typed aggregates, block-level purity annotations, and stricter contract checks so engineers can validate control flow and metadata while catching unsupported constructs early.【F:src/backend/text.rs†L1-L134】【F:src/backend/llvm.rs†L1-L420】【F:src/tests/backend_tests.rs†L1-L280】

### Tooling and Developer Experience
- The `mica` CLI routes a single parsed module through lexing, pretty-printing, checking, resolving, lowering, IR dumps, and the LLVM preview, which keeps day-to-day validation quick while we iterate on backends.【F:src/main.rs†L17-L215】
- Documentation for the CLI and snippet generator mirrors the exposed modes so contributors understand how to reproduce backend snapshots and diagnostics locally.【F:docs/modules/cli.md†L12-L60】

## Verification Snapshot
- CI mirrors the local workflow by executing `cargo build --locked`, `cargo test --locked --all-targets`, and `cargo run --quiet --bin gen_snippets -- --check`, giving quick confidence that documentation and snippets stay synchronized with the code.【F:.github/workflows/ci.yml†L1-L23】
- The test harness covers lexer, parser, lowering, IR, backend, resolver, and diagnostics suites (54 unit-style tests today), confirming every stage of the pipeline with golden expectations and negative cases.【F:src/tests/mod.rs†L1-L17】【F:src/tests/pipeline_tests.rs†L1-L139】
- Backend and pipeline tests keep the effect system, capability metadata, and match diagnostics stable while we refine IR semantics.【F:src/tests/backend_tests.rs†L1-L280】【F:src/tests/resolve_and_check_tests.rs†L1-L210】

## Phase 2 Exit Criteria
- ✅ **Aggregate modelling**: Record layouts now carry concrete offsets, size, and alignment metadata that the LLVM backend renders directly, eliminating the placeholder struct stub from earlier milestones.【F:src/ir/mod.rs†L700-L840】【F:src/backend/llvm.rs†L180-L340】【F:src/tests/backend_tests.rs†L200-L260】
- ✅ **Backend fidelity**: The LLVM emitter annotates blocks with purity information, enforces record layout availability, and surfaces errors for unsupported constructs so textual output mirrors the real codegen contract.【F:src/backend/llvm.rs†L1-L420】【F:src/tests/backend_tests.rs†L1-L280】
- ✅ **Diagnostics depth**: Semantic checks now surface duplicate capabilities, missing bindings, and scope violations with regression coverage in the test suite.【F:src/semantics/check.rs†L650-L940】【F:src/tests/resolve_and_check_tests.rs†L1-L210】
- ✅ **Purity analysis**: SSA functions include connectivity-aware purity reports that identify effect-free regions for future parallelization work.【F:src/ir/analysis.rs†L1-L140】【F:src/tests/ir_tests.rs†L280-L360】

## Next Focus Areas
1. **Native code generation**: Replace the textual LLVM preview with real module emission and link-time integration so simple programs can execute via the toolchain.【F:docs/roadmap/compiler.md†L170-L215】
2. **Runtime capability shims**: Map effect annotations to deterministic runtime providers to unlock IO/task scheduling experiments in Phase 3.【F:docs/roadmap/compiler.md†L200-L240】
3. **Concurrent metadata access**: Design thread-safe ownership for shared type/effect tables ahead of parallel backend execution.【F:src/ir/mod.rs†L500-L620】【F:docs/status.md†L40-L80】
4. **Structured diagnostics**: Extend semantic checks toward borrow flows and backend validation while maintaining the richer regression suite established in Phase 2.【F:src/semantics/check.rs†L1-L960】【F:docs/roadmap/milestones.md†L37-L60】

## Watch Items
- Shared type/effect tables in the IR will need concurrency-safe access once multiple backends consume a module; design the ownership story before parallel compilation enters the picture.【F:src/ir/mod.rs†L500-L620】
- CLI outputs are textual today; consider structured formats so tooling built during Phase 3+ can ingest resolver and IR data without fragile scraping.【F:src/main.rs†L51-L201】【F:docs/modules/cli.md†L54-L60】
