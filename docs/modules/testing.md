# Testing Harness

## Scope

The `src/tests` directory contains the end-to-end regression suite for the Mica
compiler. Each module focuses on a particular compiler stage, ensuring that
front-end, semantic, and IR behavior stay aligned with the roadmap milestones.

## Suite Overview

| File | Coverage |
| --- | --- |
| `lexer_tests.rs` | Validates tokenization of keywords, literals, and punctuation, asserting diagnostic behavior for malformed input.【F:src/tests/lexer_tests.rs†L1-L138】 |
| `parser_tests.rs` | Confirms the parser builds the correct AST for modules, imports, functions, and expressions, including error handling paths.【F:src/tests/parser_tests.rs†L1-L200】 |
| `pretty_tests.rs` | Ensures pretty-printing round-trips representative AST structures, supporting the roadmap’s lossless parsing goal.【F:src/tests/pretty_tests.rs†L1-L84】【F:docs/roadmap/compiler.md†L39-L74】 |
| `resolve_and_check_tests.rs` | Exercises resolver output (imports, capabilities, symbol scopes) and the effect checker’s diagnostics, covering roadmap Phase 1 semantics.【F:src/tests/resolve_and_check_tests.rs†L1-L220】【F:docs/roadmap/compiler.md†L76-L170】 |
| `lowering_tests.rs` | Verifies that AST constructs lower to the expected HIR forms, guarding the Phase 2 desugaring milestones.【F:src/tests/lowering_tests.rs†L1-L120】【F:docs/roadmap/compiler.md†L126-L170】 |
| `ir_tests.rs` | Checks SSA lowering output, instruction ordering, and terminators to keep backend groundwork stable.【F:src/tests/ir_tests.rs†L1-L75】【F:docs/roadmap/compiler.md†L170-L215】 |
| `display_tests.rs` | Snapshot tests for CLI display helpers to ensure human-facing diagnostics remain polished.【F:src/tests/display_tests.rs†L1-L36】 |
| `pipeline_tests.rs` | End-to-end integration tests that run the compiler pipeline across real examples, catching regressions that span multiple stages.【F:src/tests/pipeline_tests.rs†L1-L120】 |
| `helpers.rs` | Shared fixtures and helper functions that load example sources and normalize diagnostics for the other test modules.【F:src/tests/helpers.rs†L1-L19】 |

## Integration Highlights

1. The test harness imports modules from every stage, ensuring cross-stage API
   changes fail fast during development.【F:src/tests/mod.rs†L1-L16】
2. CI can run these suites to enforce roadmap quality bars, especially the targeted 95% coverage for front-end phases before backend work progresses.【F:docs/roadmap/compiler.md†L9-L74】
3. Snapshot-based tests (display, pretty, pipeline) tie directly into the
   documentation tooling via the snippet generator.【F:src/bin/gen_snippets.rs†L1-L60】

## Roadmap Alignment

- **Phase 0-1:** Lexer, parser, pretty, resolver, and checker tests guarantee the
  front-end remains stable as language features expand.【F:docs/roadmap/compiler.md†L9-L170】
- **Phase 2:** Lowering and SSA tests provide confidence in the new IR layers as
  optimization work ramps up.【F:docs/roadmap/compiler.md†L126-L215】
- **Tooling:** Consistent test output feeds into documentation and IDE tooling,
  helping maintain high-quality examples and diagnostics.【F:docs/roadmap/tooling.md†L1-L60】

## CI Command Reference

To mirror the automated pipeline locally, run the same locked commands the CI workflow executes:

1. `cargo build --locked`
2. `cargo test --locked --all-targets`
3. `cargo run --quiet --bin gen_snippets -- --check`

Executing them in order validates the compiler, the regression suites, and the documentation snippets exactly as the CI job does.【F:.github/workflows/ci.yml†L1-L23】

## Next Steps

- Expand integration tests to cover multi-module workspaces once resolver
  enhancements land.
- Add property-based tests for critical lowering and SSA transformations to catch
  corner cases.
- Integrate coverage reporting into CI to track progress toward roadmap targets.
