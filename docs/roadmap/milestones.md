# Phased Execution Milestones

> A phase-by-phase plan with entry requirements, core work, exits, and signals.

This playbook sequences the workstreams so we can ship usable artefacts after
each phase and keep sight of the longer-term language vision. Each milestone
lists entry prerequisites, core work items, exit criteria, and forward-looking
signals to check before moving on.

## Phase 0 — Foundations

- **Entry**: Vision, language sketch, and module plans approved.
- **Work Items**:
  1. Finalize lexer/token spec and parser AST schemas.
  2. Set up repo scaffolding: `cargo` workspace layout, CI skeleton, coding standards.
  3. Implement lexer with unit tests and CLI `--tokens` mode.
  4. Implement parser with snapshot tests and error recovery skeleton.
- **Exit**:
  - Parser successfully round-trips all examples.
  - CI runs lint + unit tests on every PR.
  - **Future signal**: Prototype language playground can lex/parse snippets in-browser using shared components.

## Phase 1 — Semantic Core

- **Entry**: Phase 0 exit criteria satisfied.
- **Work Items**:
  1. Build resolver with symbol tables and IDE-friendly spans.
  2. Implement Hindley–Milner inference, trait solving, and borrow checker.
  3. Enforce effect capability rules; craft negative tests.
  4. Publish diagnostics style guide and update CLI outputs.
- **Exit**:
  - `mica --check examples/*` passes (positive suite) and fails with actionable messages (negative suite).
  - Borrow checker prohibits use-after-move in integration tests.
  - **Future signal**: Constraint graphs export cleanly for potential verification tooling.

## Phase 2 — Intermediate Representation

- **Entry**: Type checker stabilizes, diagnostics accepted by DX team.
- **Work Items**:
  1. Finalize typed SSA IR with shared type/effect registries and update design docs.
  2. Hook the IR into a backend interface (`mica --ir` text emitter today, LLVM/WASM tomorrow).
  3. Expand the diagnostics playbook with positive/negative regression suites.
  4. Prototype purity analysis for `par` loops and structured tasks.
- **Exit**:
  - Lowering snapshot suite covers control flow, pattern matching, effects, and tasks.
  - CLI `mica --ir` emits typed IR that backends can consume without extra context.
  - Diagnostics regression suites stay green across resolver/checker/IR changes.
  - Purity analysis identifies effect-free regions on sample programs.
  - **Future signal**: IR contains enough provenance to correlate with runtime traces.

## Phase 3 — Backend & Runtime

- **Entry**: IR lowering stable, tests passing.
- **Work Items**:
  1. Hook IR into LLVM backend, producing native binaries.
  2. Build runtime crate with capability shims (IO, Net, Time, Task scheduler).
  3. Implement deterministic task runtime with cancellation semantics.
  4. Provide `mica build` and `mica run` commands.
- **Exit**:
  - Hello-world, CSV aggregate, and parallel sum examples compile & run with deterministic results.
  - Runtime errors surface as structured diagnostics, not panics.
  - **Future signal**: Backend abstractions demonstrate feasibility of targeting WASM/MLIR without major rewrites.

## Phase 4 — Tooling & IDE

- **Entry**: Backend emits working binaries; runtime is stable.
- **Work Items**:
  1. Ship formatter (`mica fmt`) and linter framework.
  2. Build LSP server with hover, go-to-definition, diagnostics, and code actions.
  3. Generate CLI docs and integrate into mdBook site.
  4. Harden testing infrastructure (coverage, fuzzing, xtask automation).
- **Exit**:
  - VS Code extension demo validated with live editing session.
  - CI enforces fmt, lint, tests, docs, and (once reinstated) coverage.
  - **Future signal**: LSP emits capability flow data consumable by future auditing tools.

## Phase 5 — Ecosystem Launch

- **Entry**: Tooling is developer-friendly; docs pipeline stable.
- **Work Items**:
  1. Release standard library waves S0–S2 with docs + examples.
  2. Build package manager MVP (`mica package`, registry prototype).
  3. Publish interop adapters (C ABI, Python, JS) with end-to-end demos.
  4. Finalize governance and community processes.
- **Exit**:
  - Public beta announcement with downloadable toolchain.
  - First external contributors ship PRs using documented workflow.
  - **Future signal**: Registry telemetry shows capability usage patterns to guide next-wave stdlib work.

## Phase 6 — Growth & Feedback Loops

- **Entry**: Ecosystem launched.
- **Work Items**:
  1. Expand standard library (waves S3–S4) and optional GPU capability.
  2. Collect telemetry (opt-in) on compiler performance & diagnostics clarity.
  3. Iterate on ergonomics based on RFC backlog; prioritize effect-system learnability.
- **Exit**:
  - Quarterly roadmap review produces updated documents in this folder.
  - Adoption metrics tracked (downloads, RFC throughput, issue burn-down).
  - **Future signal**: Community feeds verified case studies back into governance and roadmap updates.

---

_Update this document whenever milestones shift or new phases appear._
