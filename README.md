here’s a clean, minimal-but-complete language design I think could age well. i’ll call it **Mica** (Minimal, Industrial, Composable, Auditable).

# goals (non-negotiables)

* **Small core** you can learn in an afternoon; everything else is libraries.
* **Predictable performance** (AOT-compiled, LLVM backend; zero-cost abstractions).
* **Safety by default** (no global mutable state, explicit effects, structured concurrency).
* **Deterministic parallelism** by default; opt-in nondeterminism behind a capability.
* **Ergonomic data work** (first-class iterators, records, and enums; great pattern matching).
* **Interop first** (C ABI + “foreign tasks” for Python/JavaScript without implicit sharing).

---

## vision: building blocks for the next wave

Mica is intentionally compact today so we can grow it into something much more ambitious tomorrow. The current design is the
seed of a language that:

- **Treats reliability as an architectural concern**. Explicit effects and capabilities are the scaffolding for future
  verifiability work—model checking structured tasks, replayable IO logs, and deterministic deploys.
- **Balances human-factors and performance**. A small expression-only surface keeps cognitive load low while giving the
  compiler room for advanced inference, borrow analysis, and eventual auto-tuning of data-parallel sections.
- **Welcomes polyglot systems**. By locking in C ABI stability and message-based foreign tasks now, we can layer richer
  interop (e.g., typed Arrow pipelines, GPU capability negotiation) without breaking safety guarantees.
- **Invites auditable automation**. The same lossless AST and structured diagnostics that power the formatter today enable
  future refactoring bots, effect-driven code search, and “explainable compiler” tooling.

The roadmap doubles as a research ledger: every concrete milestone is paired with a hypothesis about what the next decade of
systems programming should feel like. We build the boring-but-essential pieces first so that later experiments—capability-safe
AI calls, deterministic ML training loops, verified governance modules—land on bedrock.

---

# core model

* **Evaluation**: call-by-value; expressions only (statements are sugar for expressions).
* **Purity**: functions are pure unless they declare effects.
* **Memory**: move semantics with borrow/refs; lifetimes inferred (no user-written lifetime syntax).
* **Types**: Hindley-Milner inference + algebraic data types (ADTs) + parametric generics + traits.
* **Effects**: every function annotates an **effect row** (e.g., `!{io, net}`) that the compiler checks.
* **Concurrency**: structured tasks; message channels; data-parallel loops for pure code.

---

# minimal surface (10 constructs)

1. literals (`42`, `3.14`, `"txt"`, `true`, `false`)
2. bindings `let`, mutable `let mut`
3. functions `fn` (lexical closures)
4. `if ... else`
5. `match` (exhaustive, on any ADT)
6. loops: `for` (iterators), `loop {}` + `break`/`continue`
7. types: `type` (aliases, records, enums), `impl` for traits
8. modules: `module`, `pub`, `use`
9. tasks: `spawn`, `await`, channels `chan`
10. effects: `!{...}` on function types + capability values

---

# syntax snapshot

```mica
module demo.csv

// algebraic data types
type Option[T] = Some(T) | None
type Result[T,E] = Ok(T) | Err(E)

// records are structural
type Row = { id: Int, qty: Int, price: F64 }

// generic function, pure
fn total[T](xs: Iterable[T], f: fn(T) -> F64) -> F64 {
  let mut s = 0.0
  for x in xs { s = s + f(x) }
  s
}

// effectful function (needs io capability)
fn read_rows(path: String, io: IO) -> Result[List[Row], String] !{io} {
  match File::open(path, io) {
    Ok(f) => {
      using f { // RAII scope, auto-close
        Ok(parse_csv(f.lines()))
      }
    }
    Err(e) => Err(e.msg)
  }
}

// structured concurrency + isolation
fn fetch_all(urls: List[String], net: Net) -> Result[List[Bytes], NetErr] !{net} {
  let tasks = urls.map(|u| spawn http::get(u, net));
  // await preserves structure; cancellation rolls up
  tasks.collect(|t| await t)?
}

// pattern matching
fn price(row: Row) -> F64 {
  row.qty as F64 * row.price
}
```

---

# types, traits, generics

* **Primitives**: `Bool, Int, I8/I16/I32/I64, U8/..., F32, F64, Char, String, Bytes`.
* **ADTs**: `enum`-style via `type Sum = A(Int)|B(String)`.
* **Records**: structural with **row polymorphism** (open records for extensibility).
* **Generics**: `fn max[T: Ord](a: T, b: T) -> T`.
* **Traits** (typeclasses): `Eq, Ord, Hash, Copy, Send`. Laws documented, not enforced by the compiler.

---

# effect system (capability-checked)

* Functions declare effect sets: `fn f(...) -> T !{io, net}`.
* Code can’t perform IO/net/random/time without a **capability value** (`IO`, `Net`, `Rand`).
* Capabilities are **linearly scoped resources** (move-only). Tests can **inject handlers** to mock effects.
* Small standard effect set: `io, net, time, rand, fs, nondet`. Everything else is library-level above `io/net`.

**Example**

```mica
fn now_ms(time: Time) -> I64 !{time} { time.millis() }
```

---

# memory & resources (simple, safe, fast)

* **Move semantics** by default. `Copy` is opt-in via trait derivation for PODs.
* **References**: `&T` (shared, immutable), `&mut T` (unique, exclusive). Lifetimes inferred.
* **RAII** with `using` blocks for guaranteed deterministic cleanup (files, sockets, GPU buffers).
* No GC in core; optional library GC for special cases (e.g., big graphs) behind `!{gc}` capability.

---

# concurrency model

* **Structured tasks**: `spawn { ... }` returns `Task[T]`; `await` only inside functions that declare `!{nondet}` *or* effectful concurrency. Pure `par` needs no nondet if the function is pure.
* **Message passing**: `let (tx, rx) = chan[T](capacity)`. Endpoints are linear; prevents use-after-close.
* **Data parallelism**: `par for x in xs { ... }` allowed only for **pure** bodies → deterministic.

**Example**

```mica
fn sum_parallel(xs: Slice[F64]) -> F64 {
  let chunks = xs.chunks(cpu::count());
  let parts  = par map chunks as |c| total(c, |x| x);
  total(parts, |x| x)
}
```

---

# errors

* **No exceptions** in core. Use `Result[T,E]` and the `?` operator to propagate.
* Effect handlers can convert OS/IO faults into typed `E`.

---

# modules & builds

* `module a.b.c` + `pub` exports. Files map 1:1 to modules.
* Reproducible builds via a single `mica.toml` manifest.
* `mica fmt`, `mica test`, `mica doc`, `mica bench` are standard tools.

---

# FFI & interop

* **C ABI**: `extern "c" fn ...`.
* **Foreign tasks**: isolate interpreters (Python/JS) in their own workers with message boundaries; pass **Bytes/JSON/Arrow** only, never raw pointers. This keeps Mica’s safety & determinism.

---

# standard library (tiny, opinionated)

* `core`: `Option, Result, List, Vec, Map, Set, String, Bytes, Iterator`.
* `math`: `vec`, `mat`, `linalg` (minimal, BLAS-backed when available).
* `sys`: `fs, net, time, process`.
* `concurrent`: `Task, Chan, Mutex` (rarely needed; prefer channels).
* `data`: CSV/JSON/Arrow readers returning **iterators**.
* Everything else lives in packages.

---

# minimal grammar (EBNF-ish)

```
Module   := "module" Path { Item }
Item     := Fn | Type | Impl | Let | Use
Fn       := "pub"? "fn" Ident TypeParams? "(" Params? ")" "->" Type EffRow? Block
EffRow   := "!{" Ident { "," Ident } "}"
Type     := Ident | Prim | "[" Type "]" | "&" Type | "&mut" Type
          | "{" Fields? "}" | Ident "[" Type { "," Type } "]"
Match    := "match" Expr "{" { Pattern "=>" Expr "," } "}"
For      := "for" Ident "in" Expr Block
Spawn    := "spawn" Expr
Await    := "await" Expr
```

---

# tiny cookbook

**hello world (pure printing via capability)**

```mica
fn main(io: IO) !{io} { io.println("hello, world") }
```

**CSV aggregate, streamed**

```mica
fn avg_price(path: String, io: IO) -> Result[F64, String] !{io} {
  let rows = read_rows(path, io)?;
  Ok(total(rows, |r| r.price) / (rows.len() as F64))
}
```

**HTTP fetch in parallel (deterministic structure)**

```mica
fn fetch_texts(urls: List[String], net: Net) -> Result[List[String], NetErr] !{net} {
  let tasks = urls.map(|u| spawn http::get(u, net));
  tasks.collect(|t| String::from_utf8(await t?)) // `?` on each result
}
```

**safe resource handling**

```mica
fn copy_file(src: String, dst: String, io: IO) -> Result[Unit, String] !{io} {
  using s = File::open(src, io)? {
    using d = File::create(dst, io)? {
      io.copy(s, d)
    }
  }
  Ok(())
}
```

---

# why this could be “the future”

The current feature set isn’t nostalgia—it’s a launchpad. Each non-negotiable is chosen because it unlocks a concrete, future
facing capability:

* **Minimal core + explicit effects** keeps the language teachable today while enabling capability-sensitive scheduling,
  replay, and formal verification later.
* **Determinism by default** makes parallel programs testable and reproducible (critical for ML/data and financial workloads);
  future compiler passes can safely auto-parallelize pure regions without surprising users.
* **Resource & capability model** gives precise control (files, nets, GPUs) without foot-guns, and sets the stage for
  capability marketplaces or audited third-party service access.
* **Interoperability boundaries** mirror real systems (safe Python/JS embedding; C where it counts) so we can explore richer
  polyglot runtimes, typed data pipes, and progressive migration stories.
* **Row-polymorphic records + ADTs** keep the type system expressive yet inference-friendly, paving the way for effect-aware
  macros, synthesis tools, and dependent-like refinements without sacrificing approachability.

Taken together, Mica aims to feel like the missing convergence point between Rust’s safety, Haskell’s purity, Erlang’s
predictability, and modern data engineering expectations.

---

# implementation roadmap

The implementation roadmap now lives in [`docs/roadmap/`](docs/roadmap/). The tables below summarize the highlights; dive into the dedicated files for detailed task breakdowns, dependencies, and exit criteria.

## implementation plan by module

| Module | Primary focus | Near-term deliverables | Detailed plan |
| --- | --- | --- | --- |
| Lexer & tokens | Streaming UTF-8 lexing with spans and incremental re-lex hooks. | Token catalogue, zero-copy cursor, snapshot tests for literals/effects. | [`compiler.md`](docs/roadmap/compiler.md#1-lexer--token-infrastructure-srclexerrs-srctokenrs) |
| Parser & AST | Recursive-descent parser with lossless mode. | Pratt table, recovery hooks, round-trip formatting tests. | [`compiler.md`](docs/roadmap/compiler.md#2-parser--ast-srcparserrs-srcastrs) |
| Resolver | Module/type/value capability tracking. | Two-phase symbol resolution, IDE lookup tables. | [`compiler.md`](docs/roadmap/compiler.md#3-resolver-srcresolverrs) |
| Type & effect checker | Hindley–Milner + traits + borrow checker. | Row-polymorphic unification, capability diagnostics, negative tests. | [`compiler.md`](docs/roadmap/compiler.md#4-type--effect-checker-srccheckrs) |
| Lowering & IR | SSA-like IR with effect metadata. | IR data model, purity analysis, snapshot suite. | [`compiler.md`](docs/roadmap/compiler.md#5-lowering--intermediate-representation-srclowerrs-srcir) |
| Backend & runtime | LLVM codegen and capability shims. | `mica build/run`, deterministic task runtime. | [`compiler.md`](docs/roadmap/compiler.md#6-backend--runtime-interface-srcbackend-runtime) |
| Formatter | Deterministic formatting & range support. | CST facade, idempotence tests, CLI integration. | [`compiler.md`](docs/roadmap/compiler.md#7-formatter--pretty-printer-srcprettyrs) |
| CLI | Unified subcommands + JSON output. | Pipeline caching, structured exit codes, snapshot tests. | [`compiler.md`](docs/roadmap/compiler.md#8-command-line-interface-srcbinmicars-srcmainrs) and [`tooling.md`](docs/roadmap/tooling.md#command-line-interface-enhancements) |
| Tooling & quality | Formatter, linting, LSP, CI automation. | Formatter/linter rules, LSP server, coverage/fuzzing. | [`tooling.md`](docs/roadmap/tooling.md) |
| Ecosystem & interop | Standard library waves, package manager, FFI. | `mica.toml` spec, registry prototype, Python/JS adapters. | [`ecosystem.md`](docs/roadmap/ecosystem.md) |

Each detailed module plan lists objectives, concrete tasks, dependencies, exit criteria, and the longer-horizon experiments we
expect to unlock when the groundwork is finished.

## phased execution guide

| Phase | Goal | Entry criteria | Exit criteria | Reference |
| --- | --- | --- | --- | --- |
| 0 — Foundations | Bootstrap lexer + parser and CI skeleton. | Vision + plans ratified. | Parser round-trips examples; CI enforces lint/tests. | [`milestones.md`](docs/roadmap/milestones.md#phase-0--foundations) |
| 1 — Semantic Core | Resolver + type/effect checker. | Phase 0 exits met. | `mica --check` passes suite; borrow checker blocks misuse. | [`milestones.md`](docs/roadmap/milestones.md#phase-1--semantic-core) |
| 2 — IR | Lowering pipeline + purity analysis. | Stable checker & diagnostics. | IR snapshots cover major constructs; purity analysis flags effect-free regions. | [`milestones.md`](docs/roadmap/milestones.md#phase-2--intermediate-representation) |
| 3 — Backend & Runtime | LLVM backend + runtime shims. | IR stable, tests green. | Native binaries for examples; structured runtime errors. | [`milestones.md`](docs/roadmap/milestones.md#phase-3--backend--runtime) |
| 4 — Tooling & IDE | Formatter, LSP, hardened testing. | Backend shipping binaries. | VS Code demo; CI matrix enforces fmt/lint/tests/docs/coverage. | [`milestones.md`](docs/roadmap/milestones.md#phase-4--tooling--ide) |
| 5 — Ecosystem Launch | Packages, interop, stdlib waves. | Tooling stable, docs pipeline live. | Public beta; external contributors landing PRs. | [`milestones.md`](docs/roadmap/milestones.md#phase-5--ecosystem-launch) |
| 6 — Growth | Feedback loops, extended libraries. | Ecosystem launched. | Quarterly roadmap reviews; adoption metrics tracked. | [`milestones.md`](docs/roadmap/milestones.md#phase-6--growth--feedback-loops) |

For tactical task lists, staffing notes, and acceptance tests per milestone, consult [`docs/roadmap/milestones.md`](docs/roadmap/milestones.md).

---

# migration story

* Write **libraries** that mirror POSIX, BLAS, Arrow, HTTP—thin and explicit.
* FFI shims for C; “foreign task” adapters for Python (`multiprocessing`) and Node (`worker_threads`).
* Starter transpilers (subset) from TypeScript (for pure/iter code) and from Rust (for ADTs/traits).

---

# what stays out (on purpose)

* No exceptions, reflection, macros, or runtime type erasure in core.
* No implicit global randomness/time.
* No hidden threads or green-thread scheduler; concurrency is explicit and structured.

---

# Trying It (Prototype Parser)

- Build: `cargo build`
- Tokens: `cargo run -- --tokens examples/demo.mica`
- AST: `cargo run -- --ast examples/demo.mica`
- Extra example: `cargo run -- --ast examples/channels.mica`

## More CLI modes

- Pretty AST: `cargo run -- --ast --pretty examples/adt.mica`
- Exhaustiveness check: `cargo run -- --check examples/adt.mica`
- Resolver dump: `cargo run -- --resolve examples/adt.mica`
- Lower to simple HIR: `cargo run -- --lower examples/methods.mica`

## Included examples

- ADTs: `examples/adt.mica`
- Using + `?`: `examples/using.mica`
- Channels: `examples/channels.mica`
- Impl and bounds: `examples/impls.mica`
- Casts and patterns: `examples/cast_and_patterns.mica`
- Methods + self receiver: `examples/methods.mica`
- Generics + bounds: `examples/generics_bounds.mica`
- Effects + using + fn type effects: `examples/effects_and_using.mica`
- Exhaustive ADT match: `examples/adt_match_exhaustive.mica`
- Spawn + await: `examples/spawn_await.mica`
- Lists + loops: `examples/lists_and_loops.mica`

## Language Guide (Prototype)

- ADTs and pattern matching
  - Define sum types with `type T = A(Int) | B(String)`.
  - Match is exhaustive; the checker warns on missing variants. See `examples/adt_match_exhaustive.mica`.
- Effects and resources
  - Functions annotate `!{io, net}`; capabilities are explicit params.
  - RAII cleanup via `using` blocks; `?` propagates `Result`. See `examples/effects_and_using.mica`.
- Concurrency primitives
  - `spawn` returns a task; `await` joins within structure. See `examples/spawn_await.mica`.
- Generics and trait bounds
  - Single bound syntax: `fn max[T: Ord](...)` and impl blocks. See `examples/generics_bounds.mica`, `examples/impls.mica`.
- Collections and loops
  - List type `[T]` and `for` loops over iterables. See `examples/lists_and_loops.mica`.
- Methods and receivers
  - `impl Trait for Type { fn f(self, ...) }`; method calls lower to simple HIR. See `examples/methods.mica` and `--lower`.

## Tests

Run the test suite: `cargo test`

What’s covered:
- Lexing of key tokens (`::`, `?`, `using`, `chan`)
- Parsing of ADTs, `using` + `?`, channel creation, casts, tuple/record patterns
- Impl blocks with `self` receivers
- Resolver mapping ADTs and variants
- Exhaustiveness checking for `match` against in-module ADTs
- Pretty-printer snapshot sanity checks
- Lowering method calls to simple HIR

## Documentation

- Tour: `docs/tour.md` — a quick walkthrough of the core language with examples.
- CLI Snippets: `docs/snippets.md` — sample outputs for `--ast --pretty`, `--check`, and `--lower`.

### Keeping docs in sync

- Regenerate CLI snippets: `cargo run --bin gen_snippets`
- Verify snippets are up-to-date (CI does this too): `cargo run --bin gen_snippets -- --check`
