# Mica Language Tour

> A rapid, example-driven overview of the language surface and tooling.

Use this tour to explore Mica. Every section includes runnable snippets from `examples/` so you can experiment with the compiler immediately.

[← Back to Documentation Home](index.html)

## Modules
- Declare one module per file with `module a.b.c`.
- Export items by marking them with `pub`.

## Types and Algebraic Data Types
```mica
module demo.adt
pub type Option[T] = Some(T) | None
pub type Result[T, E] = Ok(T) | Err(E)
```
Records are structural:
```mica
type Row = { id: Int, qty: Int, price: F64 }
```

## Pattern Matching
Patterns cover ADTs, tuples, and records. The compiler enforces exhaustiveness.
```mica
module demo.matching

type Color = Red | Green | Blue

func to_text(c: Color) -> String {
  match c { Red => "red", Green => "green", Blue => "blue" }
}
```
See `examples/adt_match_nonexhaustive.mica` for the diagnostic when a branch is
missing. For a larger pipeline that combines match guards with logging, try
`examples/concurrency_pipeline.mica`.

## Effects and `using`
Effects are explicit through effect rows and capability values.
```mica
func open_and_print(io: IO) !{io} {
  using File::open("/tmp/example.txt", io)? { io.println("opened ok") }
}

func run_with(callback: func(Int) -> Int !{io}, io: IO) !{io} {
  let _ = callback(42)
}
```
Nested scopes with higher-order helpers appear in
`examples/effects_resource_pool.mica`.

## Concurrency
Structured tasks use `spawn`/`await`.
```mica
func fetch(u: String, net: Net) -> Bytes !{net} {
  await spawn http::get(u, net)
}
```
Fan-out/fan-in pipelines with classification and logging live in
`examples/concurrency_pipeline.mica`. For a comprehensive workflow that combines
concurrency, nested `using`, and auditing, explore
`examples/comprehensive_deployment.mica`.

## Generics and Bounds
```mica
func max[T: Ord](a: T, b: T) -> T { if a < b { b } else { a } }
```
Recursive ADTs with traversal helpers appear in
`examples/generics_tree_algorithms.mica`.

## Collections and Loops
```mica
func sum(xs: [Int]) -> Int { let mut s = 0; for x in xs { s = s + x }; s }
```

## Methods and Receivers
`impl` blocks introduce receiver-based methods that lower to simple HIR when you
run `--lower`.
```mica
impl Addable for Vec2 { func add(self, other: Vec2) -> Vec2 { other } }
func use_method(a: Vec2, b: Vec2) -> Vec2 { a.add(b) }
```

## CLI Shortcuts

Explore different compiler stages:

- `--tokens` — Lex the source file
- `--ast` — Parse into an AST (default mode)
- `--ast --pretty` — Pretty-print the AST
- `--resolve` — Inspect bindings and capabilities
- `--check` — Run exhaustiveness checks
- `--lower` — Lower to the simple HIR
- `--ir` — Dump the typed SSA IR
- `--llvm` — Emit the LLVM scaffolding preview
- `--build` — Produce a native binary
- `--run` — Compile and run via the native backend

See [CLI snippets](snippets.html) for real compiler output.

---

## Next Steps

- **[Examples](https://github.com/Sir-Teo/mica/tree/main/examples)** — Runnable programs showcasing all features
- **[CLI Reference](module_reference.html)** — Deep dives into compiler modules
- **[Roadmap](roadmap/index.html)** — Future milestones and development plans
- **[Status](status.html)** — Current project health and priorities

[← Back to Documentation Home](index.html)
