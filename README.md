# Mica

<p align="center">
  <strong>Mica</strong> is an experimental systems language that explores a tiny, expressive
  core with explicit effects, deterministic concurrency, and predictable
  performance.<br />
  <em>Minimal • Industrial • Composable • Auditable</em>
</p>

Mica ships as a compact prototype compiler front-end backed by runnable
examples, language tour documentation, and snapshot-tested tooling. The project
is intentionally small enough to read in a weekend while still demonstrating the
design patterns required for a production-grade compiler.

## Highlights

- **Tiny but expressive core** – Learn the entire syntax in an afternoon and
  extend behaviour via standard libraries instead of bespoke keywords.
- **Deterministic concurrency** – Structured `spawn`/`await` with effect
  tracking ensures reproducible outcomes.
- **Capability-based effects** – Explicit `using` clauses make side effects and
  resource access auditable.
- **Ahead-of-time tooling** – The CLI exposes every compiler stage from tokens
  to native code generation, each backed by golden-file snapshots.
- **Interop-friendly ABI** – C-compatible calling conventions and hooks for
  Python/JavaScript “foreign tasks” keep the language ecosystem-friendly.

---

## Table of contents

- [Highlights](#highlights)
- [Project status](#project-status)
- [Quickstart](#quickstart)
- [CLI reference](#cli-reference)
- [Language features](#language-features)
- [Example gallery](#example-gallery)
- [Repository layout](#repository-layout)
- [Documentation & resources](#documentation--resources)
- [Development workflow](#development-workflow)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [FAQ](#faq)
- [License](#license)

## Project status

Mica is a prototype under active design. Today the repository contains:

- A lexer, parser, resolver, effect checker, lowerer, and pretty-printer wired
  together behind a single CLI binary (`mica`).
- Exhaustiveness checking for `match` expressions, capability tracking, and
  structured diagnostics with source-code snippets.
- Snapshot-driven documentation for the CLI, plus a tutorial-oriented language
  tour.
- An executable test suite covering lexing, parsing, resolving, lowering,
  formatting, and both textual/native backend paths.

See the [roadmap](#roadmap) for the longer-term build-out and current areas of
focus.

## Quickstart

### 1. Install prerequisites

- Install a recent stable Rust toolchain via <https://rustup.rs/>.
- LLVM is bundled with Rust, so no additional native dependencies are required
  for the current prototype.

### 2. Clone and build

```bash
git clone https://github.com/<you>/mica.git
cd mica
cargo build
```

The initial build downloads the Rust dependencies. Subsequent builds and runs
are incremental and fast.

### 3. Run the test suite

```bash
cargo test
```

The tests execute integration flows that cover lexing, parsing, lowering, the
IR pipeline, and snapshot comparisons.

### 4. Explore the CLI

Inspect the AST of a demo program:

```bash
cargo run --bin mica -- examples/demo.mica
```

Or compile and execute one of the runnable samples end-to-end:

```bash
cargo run --bin mica -- --run examples/methods.mica
```

### 5. Troubleshooting

- If builds fail due to an outdated toolchain, run `rustup update` and retry.
- Regenerate CLI snapshots when golden files drift: `cargo run --bin gen_snippets`.
- Use `cargo clean` to clear stale build artifacts when switching toolchains.

## CLI reference

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

### Editor integration

A dedicated language server is still in development. In the meantime, the
pretty-printer, CLI snapshots, and `cargo fmt` keep code style consistent.

## Language features

### Quick example

The following snippet demonstrates pattern matching and capability-aware
logging. Save it as `hello.mica` and run `cargo run --bin mica -- --run hello.mica`:

```mica
module hello

capability Console

func main(using Console console) -> Result[Int, Error] {
  let hour = 17
  let message = match hour {
    hour if hour < 12 -> "Good morning"
    hour if hour < 18 -> "Good afternoon"
    _                 -> "Good evening"
  }

  console.println(message)
  Ok(0)
}
```

The example highlights Mica's explicit capabilities and exhaustive `match`
expressions, even in small standalone modules.

### Feature highlights

- **Modules & namespaces** — `module` declarations map directly to file system
  layout, simplifying code navigation.
- **Algebraic data types (ADTs)** — First-class enums with pattern matching and
  guard clauses.
- **Deterministic concurrency** — `spawn`, `await`, and effect-scoped `using`
  blocks keep asynchronous flows auditable.
- **Generics & traits** — Zero-cost abstractions with trait-constrained
  generics and higher-order functions.
- **Capabilities & effects** — Resources are granted explicitly, keeping IO,
  logging, and networking opt-in.

## Example gallery

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
capabilities compose across files. Pair them with the [language tour](#documentation--resources)
for guided explanations.

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
the pipeline quickly. Every directory contains focused Rust modules with
documented entry points to make exploratory reading easier.

## Documentation & resources

- [`docs/tour.md`](docs/tour.md) — Guided walkthrough of syntax, semantics, and
  runtime behaviour.
- [`docs/snippets.md`](docs/snippets.md) — CLI output snapshots that double as
  regression tests.
- [`docs/roadmap/`](docs/roadmap) — Milestones, design notes, and implementation
  plans.
- [`examples/`](examples) — Runnable programs referenced throughout the tour
  and used in integration tests.
- [Issues](https://github.com/zesterer/mica/issues) — Discussion of active work,
  feature requests, and bug reports.

## Development workflow

1. Make changes to the compiler or examples.
2. Format and lint your work:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features
   ```
3. Run the full test suite:
   ```bash
   cargo test
   ```
4. Update CLI snapshots when relevant:
   ```bash
   cargo run --bin gen_snippets -- --check   # verify
   cargo run --bin gen_snippets              # regenerate if needed
   ```

Continuous integration mirrors this workflow, so keeping the tree clean ensures
fast reviews.

### Recommended tooling

- `cargo check` for quick iteration loops.
- `cargo fmt --check` in CI environments to verify formatting.
- An editor with rust-analyzer or language-server support for Rust will surface
  type errors as you work on the compiler.

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
- Open an issue to discuss ideas, questions, or areas where you would like to
  help.

Community discussion channels (such as GitHub Discussions) will come online as
the project matures—watch the repository for updates.

## FAQ

### Is Mica production ready?

Not yet. The language is still iterating on core semantics and tooling. Expect
breaking changes between releases.

### Does Mica target a virtual machine or native code?

Mica lowers to a typed SSA IR and emits LLVM IR for native code generation. A
bytecode or WASM backend may appear in future roadmap milestones.

### Where can I ask questions or follow along?

Track issues and discussions in the GitHub repository, or open an issue to start
a conversation. News about community chat channels will land in the roadmap
directory as they become available.

## License

This prototype does not yet declare a license. Until one is added, please reach
out to the maintainers before using Mica in production settings.
