# Mica

Mica is an experimental systems programming language exploring a tiny, expressive
core with predictable performance, explicit effects, and deterministic
concurrency. The repository currently ships a prototype compiler front-end with
a growing suite of language services, documentation, and examples.

- **Minimal core** you can learn in an afternoon—everything else is library
  code.
- **Predictable performance** via ahead-of-time compilation and zero-cost
  abstractions.
- **Safety and auditability** through explicit capabilities, structured
  concurrency, and deterministic parallelism.
- **Interop-first design** with a stable C ABI and hooks for Python/JavaScript
  "foreign tasks".

> Mica stands for **Minimal, Industrial, Composable, Auditable**.

## Table of contents

- [Project status](#project-status)
- [Getting started](#getting-started)
- [Command-line interface](#command-line-interface)
- [Language tour](#language-tour)
- [Examples](#examples)
- [Repository layout](#repository-layout)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Project status

Mica is a prototype under active design. Today the repository contains:

- A lexer, parser, resolver, effect checker, lowerer, and pretty-printer wired
  together behind a single CLI binary (`mica`).
- Exhaustiveness checking for `match` expressions, capability tracking, and
  structured diagnostics.
- Snapshot-driven documentation for the CLI, plus a tutorial-oriented language
  tour.
- An executable test suite covering lexing, parsing, resolving, lowering,
  formatting, and both textual/native backend paths.

See the [roadmap](#roadmap) for the longer-term build-out.

## Getting started

Mica uses the Rust toolchain. Install Rust via <https://rustup.rs/> if you do not
already have it, then build and test the project:

```bash
cargo build
cargo test
```

Running `cargo run --bin mica` with the path to a `.mica` file executes the CLI
in its default `--ast` mode:

```bash
cargo run --bin mica -- examples/demo.mica
```

## Command-line interface

The CLI surfaces multiple compiler stages behind feature flags. Combine them as
needed:

```bash
cargo run --bin mica -- --tokens examples/demo.mica      # Lex the source file
cargo run --bin mica -- --ast examples/demo.mica         # Parse into an AST (default mode)
cargo run --bin mica -- --ast --pretty examples/adt.mica # Pretty-print the AST
cargo run --bin mica -- --resolve examples/adt.mica      # Inspect bindings and capabilities
cargo run --bin mica -- --check examples/adt.mica        # Exhaustiveness checks
cargo run --bin mica -- --lower examples/methods.mica    # Lower to the simple HIR
cargo run --bin mica -- --ir examples/methods.mica       # Dump the typed SSA IR via the backend shim
cargo run --bin mica -- --llvm examples/methods.mica     # Emit the LLVM scaffolding preview
cargo run --bin mica -- --build examples/methods.mica    # Produce a native binary next to the source
cargo run --bin mica -- --run examples/methods.mica      # Compile + run via the native backend
```

CLI output snapshots are maintained in [`docs/snippets.md`](docs/snippets.md).

## Language tour

For a quick walkthrough of modules, algebraic data types, pattern matching,
effects, concurrency, generics, and more, read [`docs/tour.md`](docs/tour.md).
Each section in the tour links directly to runnable examples in the
[`examples/`](examples) directory.

## Examples

The `examples/` directory showcases the current surface of the language,
including:

- `adt.mica` — defining algebraic data types and matching over them.
- `concurrency_pipeline.mica` — coordinated `spawn`/`await` workflows, pattern
  matching, and capability-aware logging.
- `effects_and_using.mica` — explicit capabilities with RAII `using` blocks.
- `effects_resource_pool.mica` — higher-order helpers that combine `using`,
  structured error propagation, and cross-capability tasks.
- `comprehensive_deployment.mica` — an end-to-end workflow that mixes
  concurrency, nested `using` scopes, and result propagation across a deployment
  plan.
- `native_entry.mica` — a minimal module with an entry point that exercises the
  native `--build`/`--run` workflow end-to-end.
- `generics_tree_algorithms.mica` — recursive ADTs with higher-order generic
  functions and trait bounds in action.
- `lists_and_loops.mica` — collections, loops, and iteration patterns.
- `methods.mica` — `impl` blocks and method receivers.

Use the CLI commands above to inspect each example and explore how the
capabilities compose across files.

## Repository layout

```
├── Cargo.toml           # Crate metadata for the prototype compiler
├── src/
│   ├── main.rs          # CLI entry point and mode selection
│   ├── syntax/          # Lexer and parser
│   ├── semantics/       # Resolver, effect checker, and type utilities
│   ├── lower/           # Lowering to a simplified HIR
│   ├── backend/         # Backend traits, textual emitters, and native codegen
│   ├── diagnostics/     # Shared error and warning types
│   └── pretty/          # Concrete syntax tree formatter
├── docs/
│   ├── tour.md          # Guided tour of the language
│   ├── snippets.md      # CLI snapshot outputs
│   └── roadmap/         # Detailed design and milestone plans
└── examples/            # Sample Mica programs used in docs and tests
```

This layout is intentionally compact so newcomers can find the relevant stage of
the pipeline quickly.

## Roadmap

The roadmap in [`docs/roadmap/`](docs/roadmap) tracks milestones from
foundational parsing work through backend code generation, tooling, and
ecosystem maturity. Each milestone includes objectives, dependencies, and exit
criteria so contributors can see how prototype work connects to the larger
vision.

Highlights include:

- **Phase 0 – Foundations:** bootstrap the lexer, parser, and CI skeleton.
- **Phase 1 – Semantic Core:** resolver, type/effect checker, and borrow rules.
- **Phase 2 – IR:** lowering pipeline, purity analysis, and snapshot testing.
- **Phase 3 – Backend & Runtime:** LLVM code generation and deterministic task
  runtime.
- **Phase 4 – Tooling & IDE:** formatter, LSP server, and hardened testing.
- **Phase 5+ – Ecosystem:** package manager, FFI, and community growth.

See [`docs/roadmap/milestones.md`](docs/roadmap/milestones.md) for details.

## Contributing

Contributions are welcome! A few tips:

- Run `cargo fmt` and `cargo test` before submitting a change.
- Update snapshots with `cargo run --bin gen_snippets` when CLI output changes
  (use `-- --check` to verify they are current).
- Prefer small, focused pull requests so reviews stay fast and friendly.

Feel free to open an issue to discuss ideas, questions, or areas where you would
like to help.

## License

This prototype does not yet declare a license. Until one is added, please reach
out to the maintainers before using Mica in production settings.
