# CLI and Developer Tooling

> `mica` and `gen_snippets` make the compiler explorable from the command line
> while keeping the documentation trustworthy.

## Overview

Two binaries ship in this repository:

- `mica` — the main CLI that exposes each compiler stage behind feature flags.
- `gen_snippets` — a helper used in CI to regenerate or verify documentation
  snippets.

Together they provide a fast feedback loop for day-to-day development and ensure
that the written guides never drift from real compiler output.

## Responsibilities

- **Argument parsing** – Flags such as `--tokens`, `--ast`, `--pretty`,
  `--resolve`, `--check`, `--lower`, `--ir`, `--ir-json`, `--llvm`, `--build`, and
  `--run` map directly to compiler stages. The CLI validates that one mode is
  selected and that an input path is provided.
- **Mode dispatch** – A central enum encapsulates each stage so new passes can be
  wired in with a single match arm.
- **Pipeline execution** – Modes share the same parsing step and then call into
  the relevant modules to print tokens, ASTs, diagnostics, lowered HIR, backend
  dumps, or native build artefacts.
- **Error reporting** – Diagnostic helpers normalise error formatting, giving
  contributors consistent output across commands.
- **Documentation snapshots** – `gen_snippets` executes curated commands against
  `examples/` and either updates `docs/snippets.md` or verifies it in `--check`
  mode for CI.

## Working with the CLI

1. Run `cargo run --bin mica -- --help` to see the current set of modes.
2. Use `--resolve-json` or `--ir-json` when experimenting with tooling
   integrations—these outputs are designed to be machine-consumable.
3. Regenerate documentation snippets with `cargo run --bin gen_snippets` after
   modifying CLI output or examples. Pair it with `--check` in CI to keep the
   docs in sync.

## Roadmap Alignment

- **Front-end polish** – Early phases rely on lexing, parsing, and pretty-print
  modes to validate syntax and tutorial material.
- **Static analysis** – `--check` and `--resolve` surface effect checking and
  resolver output in preparation for advanced diagnostics.
- **Lowering groundwork** – `--lower`, `--ir`, and `--llvm` expose the HIR and
  backend contracts while integrations mature.
- **Backend and ecosystem tooling** – `--build` and `--run` exercise the native
  backend, and the structured mode enum leaves room for future optimisation or
  package management commands.

## Next Steps

- Extend the CLI help text with more examples as new passes land.
- Offer structured output for additional modes to support forthcoming language
  server work.
- Explore watch-mode or incremental recompilation hooks alongside incremental
  front-end improvements.
