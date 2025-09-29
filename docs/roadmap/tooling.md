# Tooling & Developer Experience Plan

This plan covers everything around the core compiler that improves day-to-day ergonomics while laying a foundation for
automation-heavy, next-decade developer workflows.

## Command-Line Interface Enhancements

| Stage | Deliverable | Details | Owner |
| --- | --- | --- | --- |
| T1 | Unified subcommand interface | Replace ad-hoc flags with `mica <command>` verbs. Share pipeline caches and diagnostics renderer. | Compiler team |
| T2 | JSON output surfaces | Emit machine-readable diagnostics (`--json`), AST dumps, and IR dumps for integration tests. | Compiler + DX |
| T3 | Project manifest support | Parse `mica.toml`, resolve package graph, invoke compiler per target. Introduce `mica check`, `mica build`, `mica run`. | Build systems |
| T4 | Scripting hooks | Allow `mica run --watch` and custom pipeline steps for codegen/formatting. | DX |

**Acceptance Criteria**
- CLI returns structured exit codes (0 success, 1 diagnostics, 2 internal errors).
- Snapshot tests in `tests/cli/` cover both text and JSON output.

**Future-facing trajectory**
- Run the CLI in watch/daemon modes for remote build farms and cloud editors.
- Emit provenance metadata so CI/CD systems can attest to which compiler phases ran.
- Allow declarative pipelines that enterprises can extend with capability gates.

## Formatter & Linter

1. Document canonical style in `docs/style.md` (indentation, trailing commas, effect row ordering).
2. Implement formatter (`mica fmt`) per `compiler.md` plan.
3. Introduce lint framework with passes:
   - Unused capability parameters.
   - Non-exhaustive `match` on closed enums.
   - Blocking calls inside `!{nondet}` functions.
4. Add `mica lint --allow`/`--deny` CLI for toggling rules.

**Acceptance Criteria**
- Formatting is idempotent (CI ensures zero diff on second run).
- Lint warnings integrate with JSON diagnostics and can be suppressed via `#[allow(rule)]` attributes (parser support required).

**Future-facing trajectory**
- Share lint facts with the checker so the language can one day offer machine-verified rewrites.
- Support “explain this lint” documentation linking to governance-approved style rationales.
- Provide partial-formatting strategies optimized for pair-programming or literate programming contexts.

## Language Server Protocol (LSP)

1. Stand up `crates/mica-lsp/` using `tower-lsp`.
2. Wire resolver/type-checker to provide:
   - Hover (type + effect info).
   - Go-to-definition / references.
   - Diagnostics with ranges.
   - Code actions (insert `using`, add missing capability parameter).
3. Support incremental analysis using resolver’s incremental hooks.
4. Add range formatting and on-type formatting using formatter engine.
5. Integrate with VS Code extension stub (`editors/vscode/`).

**Acceptance Criteria**
- `code --enable-proposed-api mica` extension demo works on sample workspace.
- LSP integration tests simulate editing sessions and assert diagnostics churn stays under 100ms per change.

**Future-facing trajectory**
- Stream semantic graphs to collaborative IDEs for multi-user editing.
- Surface capability flow visualizations so teams can audit effect usage live.
- Expose language-server plugins that experiment with deterministic AI copilots without compromising safety.

## Documentation Workflow

1. Maintain single-source docs under `docs/` with `mdbook` or `mkdocs` (decide in T1).
2. Auto-generate CLI reference (`mica --help` -> Markdown) and embed in docs build.
3. Publish architecture notes from `compiler.md` as part of the docs site.
4. Provide “playground” instructions for running snippets.

**Acceptance Criteria**
- `cargo xtask doc` builds the site locally.
- Deployment pipeline pushes docs on main branch updates.

**Future-facing trajectory**
- Generate traceable docs (linking code, tests, roadmap) for compliance-heavy environments.
- Embed runnable sandboxes backed by deterministic task runtimes.
- Enable custom doc “lenses” (security, performance, pedagogy) derived from roadmap metadata.

## Testing Infrastructure

1. Add `cargo llvm-cov` support and enforce coverage thresholds.
2. Wire fuzzers: `cargo fuzz run lexer` and `cargo fuzz run parser` on nightly schedule.
3. Provide regression suite harness for effect system edge cases.
4. Document triage process for test flakes (see `docs/CONTRIBUTING.md`).

**Acceptance Criteria**
- CI matrix runs tests, fmt, lint, coverage, fuzz smoke, docs.
- Nightly pipeline reports coverage/fuzz metrics to Slack.

**Future-facing trajectory**
- Feed telemetry into roadmap dashboards for data-driven prioritization.
- Provide self-healing workflows (auto-open issues, attach reduced repros) when fuzzers crash.
- Experiment with prover-assisted regression tests for the effect system and borrow checker.

---

_Last updated alongside `compiler.md`._
