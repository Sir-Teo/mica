# Compiler Module Plans

> High-level goals for each compiler subsystem, plus detailed checklists below.

Use the quick reference table to orient yourself, then dive into the module
sections for step-by-step plans.

| Module | Focus | Primary Phase |
| --- | --- | --- |
| Lexer & Tokens | Streaming lexing with precise spans | Phase 0 |
| Parser & AST | Recoverable recursive-descent parser | Phase 0 |
| Resolver | Symbol graphs, capabilities, imports | Phase 1 |
| Type & Effect Checker | Inference, borrow rules, diagnostics | Phase 1 |
| Lowering & IR | HIR + SSA groundwork | Phase 2 |
| Backend & Runtime | Code generation and capability shims | Phase 3 |
| Formatter | Deterministic pretty-printing | Phase 0+ |
| CLI | Developer workflows and integrations | Phase 0+ |

Each module plan includes objectives, detailed tasks, dependencies, and exit
criteria. Work through the modules roughly in the order listed; later stages
assume the prior ones are code-complete and tested.

## 1. Lexer & Token Infrastructure (`src/lexer.rs`, `src/token.rs`)

**Objectives**
- Produce a streaming lexer that tokenizes UTF-8 source with precise span data.
- Support incremental re-lexing for IDE tooling.

**Step-by-step plan**
1. Catalogue keywords, operators, and literal forms based on the 10-construct minimal surface.
2. Implement a zero-copy cursor over byte slices with explicit error tokens for malformed UTF-8 and unterminated literals.
3. Attach `Span { file_id, offset, len }` metadata to every token.
4. Expose an incremental entry point `fn relex(range: ByteRange)` that yields token replacements for the editor pipeline.
5. Write snapshot tests covering:
   - Numeric literal edges (underscores, hex, binary, floats without integer parts).
   - Nested block comments and doc comments.
   - Capability annotations (`!{io, net}`) and `using` scopes.

**Dependencies**
- None; this is the bootstrap stage.

**Exit Criteria**
- `cargo test -p lexer` (or equivalent module tests) passes with 95%+ branch coverage for token classification.
- CLI mode `mica --tokens` renders spans and lexemes for sample programs.

**Future-facing trajectory**
- Enrich doc-comment handling to feed future doc generators and literate tooling.
- Expose token change deltas over language-server channels to support collaborative editing and remote builds.
- Prepare for capability-sensitive lexing (e.g., gated keywords) by isolating feature flags in the token catalogue.

## 2. Parser & AST (`src/parser.rs`, `src/ast.rs`)

**Objectives**
- Maintain a hand-written recursive-descent parser with Pratt expression parsing.
- Produce a recoverable AST suitable for formatting and tooling.

**Step-by-step plan**
1. Map the grammar into AST enums/structs, including effect rows, `using` blocks, trait bounds, and task constructs.
2. Implement Pratt precedence tables for arithmetic, comparison, logical, and pipeline operators (if added later).
3. Add single-token insertion/deletion recovery hooks so downstream phases still run.
4. Support a lossless mode that retains comments/whitespace for pretty-printing.
5. Expand parser tests:
   - Round-trip snapshots (`parse` -> `pretty` -> `parse`) to guarantee idempotence.
   - Error-recovery fixtures verifying diagnostic quality (expected tokens, inserted tokens).

**Dependencies**
- Lexer tokens and spans.

**Exit Criteria**
- `mica --ast --pretty` matches golden files in `tests/parse_snapshots`.
- Parser can continue after at least 3 representative syntax errors.

**Future-facing trajectory**
- Power AST-driven refactoring bots by expanding the lossless tree with stable node IDs.
- Enable partial parsing for on-device development (low-memory) by formalizing chunked parse boundaries.
- Keep the grammar adaptable for future features like effect handlers or deterministic async sugar without breaking recoverability.

## 3. Resolver (`src/resolve.rs`)

**Objectives**
- Build symbol tables for modules, types, values, traits, and capabilities.
- Track effect aliases and import visibility.

**Step-by-step plan**
1. Implement a two-phase resolver: declaration collection, then use resolution.
2. Model nested modules with lexical scopes, supporting `use` + aliasing.
3. Track capabilities/effect rows so we can flag missing capability parameters early.
4. Emit IDE-friendly lookup structures (symbol IDs + spans) shared with the LSP server.
5. Add tests covering shadowing, cross-module imports, and capability alias resolution.

**Dependencies**
- Parser AST.

**Exit Criteria**
- `mica --resolve` emits symbol graphs for examples.
- Name resolution handles at least two modules with cyclic `use` patterns (erroring gracefully).

**Future-facing trajectory**
- Persist symbol graphs for “explain this code” tooling and governance audits.
- Provide streaming symbol updates to the LSP to unlock large-workspace scaling.
- Keep capability resolution modular so future capability providers (GPU, AI, ledger) plug in cleanly.

## 4. Type & Effect Checker (`src/check.rs`)

**Objectives**
- Perform Hindley–Milner inference with trait constraints and row polymorphism.
- Enforce move semantics, borrow rules, and capability usage.

**Step-by-step plan**
1. Implement unification that supports record and effect-row extension.
2. Add trait solving with coherence checks (Rust-like orphan rules) and dictionary passing for codegen.
3. Track ownership transitions: moves, `&` borrows, `&mut` exclusivity, and `using` drop scopes.
4. Integrate capability verification: ensure functions consuming effects receive appropriate parameters.
5. Produce structured diagnostics with suggestions (`add using`, `pass io capability`).
6. Build regression tests for borrow errors, trait resolution, and effect mismatches.

**Dependencies**
- Resolver symbol tables.

**Exit Criteria**
- `mica --check` succeeds on all examples and fails with actionable diagnostics on crafted negative tests.
- Borrow checker forbids use-after-move in integration tests.

**Future-facing trajectory**
- Record proof artifacts (constraints, capability flows) for optional verification/attestation pipelines.
- Expose effect constraints to optimization passes for deterministic scheduling research.
- Structure diagnostics so future assistants can propose verified fixes without guessing intent.

## 5. Lowering & Intermediate Representation (`src/lower.rs`, `src/ir/*`)

**Objectives**
- Lower typed AST into an SSA-like IR annotated with effects.
- Prepare ground for LLVM code generation and optimization.

**Step-by-step plan**
1. Design the IR data model (modules, functions, basic blocks, effect metadata) under `src/ir/`.
2. Translate expressions/statements into SSA form with phi nodes and explicit resource drops.
3. Mark pure regions to enable deterministic auto-parallelization of `par` constructs.
4. Emit textual dumps for debugging (`mica --lower --pretty`).
5. Seed optimization passes: dead code elimination, effect-aware inlining guards.
6. Write lowering tests that compare IR snapshots for representative language features.

**Dependencies**
- Type-checked AST.

**Exit Criteria**
- IR snapshot tests live in `tests/lower_snapshots` and pass.
- Pure-region analysis recognizes at least loops/functions without effect rows.

**Future-facing trajectory**
- Tag IR nodes with provenance so auto-tuners can relate runtime metrics back to source constructs.
- Keep effect metadata first-class to experiment with deterministic distributed execution.
- Design IR serialization for future incremental compilation and remote caching services.

## 6. Backend & Runtime Interface (`src/backend/*`, `runtime/`)

**Objectives**
- Generate machine code via LLVM.
- Provide runtime shims for capabilities (IO, Net, Time) and task scheduling.

**Step-by-step plan**
1. Scaffold `src/backend/llvm.rs` with module/function emitters.
   - ✅ A transitional native backend now emits portable C (`src/backend/native.rs`) and links binaries via the host toolchain, providing an end-to-end path while the direct LLVM integration matures.【F:src/backend/native.rs†L1-L400】
2. Map capability usage to runtime calls; define `runtime/` crate for host services.
3. Implement deterministic scheduler for structured tasks, respecting cancellation semantics.
4. Hook build pipeline into `mica build` CLI command.
5. Add integration tests using `cargo test -p runtime` and golden output tests (`mica run examples/...`).

**Dependencies**
- Stable IR and effect annotations.

**Exit Criteria**
- Simple programs (hello world, parallel sum) compile to native binaries and run.
- Capability misuse at runtime produces structured errors, not panics.

**Future-facing trajectory**
- Allow backend substitution (e.g., MLIR, WASM) without reworking lowering semantics.
- Instrument runtime hooks for deterministic replay and time-travel debugging.
- Define capability provider interfaces that third parties can implement safely.

## 7. Formatter & Pretty Printer (`src/pretty.rs`)

**Objectives**
- Provide deterministic formatting consistent with the grammar and style guide.
- Support both full-file formatting and range formatting for editor support.

**Step-by-step plan**
1. Define formatting rules for declarations, match arms, effect rows, and nested `using` blocks.
2. Implement a concrete syntax tree (CST) facade around the lossless parser mode.
3. Provide `fmt::Session` APIs for whole-file and range-based formatting.
4. Build idempotence tests: format -> format again -> diff == empty.
5. Integrate with CLI (`mica fmt`) and expose `--check` mode for CI.

**Dependencies**
- Lossless parser.

**Exit Criteria**
- `mica fmt` runs on the repo without diffs after the second pass.
- Range-format tests cover typical IDE scenarios (format selection, single function).

**Future-facing trajectory**
- Share formatting primitives with lint and auto-refactor tools to keep automation trustworthy.
- Surface “format with intent” options for generated code or tutorials without drifting from canonical style.
- Lay groundwork for effect-aware diff minimization (only rewrite regions impacted by diagnostics).

## 8. Command-Line Interface (`src/bin/mica.rs`, `src/main.rs`)

**Objectives**
- Expose compiler stages via subcommands.
- Provide diagnostics and exit codes suitable for CI.

**Step-by-step plan**
1. Implement subcommands: `tokens`, `ast`, `resolve`, `check`, `lower`, `fmt`, `build`, `run`.
2. Share a pipeline builder that caches intermediate results between subcommands.
3. Add JSON output option for integration with editors and build tools.
4. Write CLI snapshot tests using `assert_cmd` for success and failure modes.

**Dependencies**
- Underlying compiler modules.

**Exit Criteria**
- CLI docs (`docs/snippets.md`) are generated automatically and verified in CI.
- Non-zero exit codes propagate for diagnostic failures.

**Future-facing trajectory**
- Offer daemon mode for build servers and cloud IDEs to minimize cold-start costs.
- Provide declarative pipelines so organizations can extend the CLI without forking.
- Integrate with future governance tooling (e.g., compile with RFC gating) via structured command metadata.

## 9. Testing & Quality Infrastructure (`tests/`, `xtask/`)

**Objectives**
- Ensure broad coverage and reproducibility across compiler modules.

**Step-by-step plan**
1. Stand up `xtask` utilities for snapshot regeneration and fixture scaffolding.
2. Reintroduce portable code coverage reporting in CI (prefer `cargo llvm-cov` when feasible, or an equivalent collector).
3. Automate fuzzing hooks for lexer/parser (`cargo fuzz` integration).
4. Document testing strategy in `docs/roadmap/tooling.md`.

**Dependencies**
- Cross-cutting; evolves with modules.

**Exit Criteria**
- CI pipeline enforces formatting, clippy, tests, snippet freshness, and reinstated coverage thresholds.
- Fuzzers execute nightly with budgeted runtime.

**Future-facing trajectory**
- Publish anonymized diagnostic telemetry (opt-in) to inform language design and documentation.
- Automate bisecting of fuzz failures with deterministic reproduction harnesses.
- Expose coverage gaps to the roadmap so planning stays data-driven.

---

*Last updated: keep this date aligned with the latest roadmap revisions.*
