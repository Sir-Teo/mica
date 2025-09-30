# CLI and Developer Tooling

## Scope

This guide covers the `mica` command-line binary (`src/main.rs`) and the
companion documentation helper `gen_snippets` (`src/bin/gen_snippets.rs`). These
executables expose the compiler pipeline for day-to-day development and keep the
written guides synchronized with reality.

## Current Responsibilities

| Area | Description |
| --- | --- |
| Argument parsing | `run()` parses flags such as `--tokens`, `--ast`, `--pretty`, `--check`, `--resolve`, `--lower`, the textual `--ir` dump, and the LLVM preview `--llvm`, validating that exactly one input path is supplied.【F:src/main.rs†L17-L63】 |
| Mode dispatch | The `Mode` enum enumerates each compiler stage that can be surfaced through the CLI and makes it trivial to wire additional passes as roadmap milestones land.【F:src/main.rs†L207-L215】 |
| Pipeline execution | Each mode reuses the same parse step and then calls into the relevant library module to print tokens, ASTs, semantic diagnostics, resolver output, lowered HIR strings, or backend dumps.【F:src/main.rs†L63-L204】 |
| Error reporting | CLI errors are normalized through the shared diagnostics helpers so messages remain consistent across modules.【F:src/main.rs†L35-L47】 |
| Documentation snapshots | `gen_snippets` rebuilds the project, executes curated commands over `examples/`, and updates `docs/snippets.md` or verifies it in `--check` mode for CI.【F:src/bin/gen_snippets.rs†L1-L60】 |

## Key Data Structures and APIs

- `Mode`: Centralized enumeration of exposed stages; follow the established
  pattern when adding backend or optimization passes from Phase 3 of the
  compiler roadmap.【F:src/main.rs†L207-L215】【F:docs/roadmap/compiler.md†L126-L215】
- `resolve::ResolvedModule`: Data printed by `--resolve`, including module path,
  imports, symbol metadata, capabilities, and resolved paths. It is an important
  integration surface for forthcoming IDE tooling.【F:src/main.rs†L75-L175】【F:docs/roadmap/tooling.md†L1-L60】
- Snapshot harness helpers in `gen_snippets`: Guarantee the docs remain accurate
  by comparing generated output against committed snippets, a prerequisite for
  the roadmap’s documentation quality goals.【F:src/bin/gen_snippets.rs†L24-L58】【F:docs/roadmap/tooling.md†L32-L60】

## Interactions with Other Modules

1. The CLI always reads sources once, lexes and parses through `mica::parser`,
   and then branches to semantic, lowering, or IR modules as necessary.【F:src/main.rs†L58-L200】
2. The pretty-printer is used for `--pretty` output, ensuring round-tripping of
   ASTs documented in the syntax roadmap.【F:src/main.rs†L56-L63】【F:docs/roadmap/compiler.md†L39-L74】
3. Resolver and capability information is surfaced to prepare for language
   server integrations described in the tooling roadmap.【F:src/main.rs†L75-L175】【F:docs/roadmap/tooling.md†L1-L60】

## Roadmap Alignment

- **Phase 0 (Front-end polish):** Existing flags satisfy the roadmap by exposing
  lexing, parsing, and pretty-printing for manual validation and tutorial
  generation.【F:docs/roadmap/compiler.md†L9-L74】
- **Phase 1 (Static analysis):** The `--check` and `--resolve` modes already pipe
  through type/effect checks and resolver output, providing scaffolding for more
  advanced diagnostics and borrow checking planned in this phase.【F:docs/roadmap/compiler.md†L76-L125】
- **Phase 2 (Lowering groundwork):** `--lower`, `--ir`, and `--llvm` expose both the HIR and the backend contracts, easing debugging as backend integrations mature.【F:src/main.rs†L184-L201】【F:docs/roadmap/compiler.md†L126-L170】
- **Phase 3 (Backend and ecosystem tooling):** The structured `Mode` enum and
  snapshot tooling leave room to add code generation, optimization, and package
  manager commands outlined for later phases.【F:docs/roadmap/compiler.md†L170-L215】【F:docs/roadmap/ecosystem.md†L1-L78】

## Next Steps

- Extend the CLI help text and usage examples once additional passes are
  implemented so users can quickly discover new capabilities.
- Integrate structured (JSON/`serde`) output for `--resolve` to feed the planned
  language server and IDE tooling efforts.【F:docs/roadmap/tooling.md†L20-L60】
- Consider adding watch-mode or incremental recompilation hooks alongside the
  roadmap’s incremental front-end work.
