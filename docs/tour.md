# Mica Tour (Prototype)

> A rapid, example-driven overview of the language surface and tooling.

Use this tour to explore the prototype. Every section includes runnable snippets
from `examples/` so you can experiment with the compiler immediately.

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

fn to_text(c: Color) -> String {
  match c { Red => "red", Green => "green", Blue => "blue" }
}
```
See `examples/adt_match_nonexhaustive.mica` for the diagnostic when a branch is
missing. For a larger pipeline that combines match guards with logging, try
`examples/concurrency_pipeline.mica`.

## Effects and `using`
Effects are explicit through effect rows and capability values.
```mica
fn open_and_print(io: IO) !{io} {
  using File::open("/tmp/example.txt", io)? { io.println("opened ok") }
}

fn run_with(callback: fn(Int) -> Int !{io}, io: IO) !{io} {
  let _ = callback(42)
}
```
Nested scopes with higher-order helpers appear in
`examples/effects_resource_pool.mica`.

## Concurrency
Structured tasks use `spawn`/`await`.
```mica
fn fetch(u: String, net: Net) -> Bytes !{net} {
  await spawn http::get(u, net)
}
```
Fan-out/fan-in pipelines with classification and logging live in
`examples/concurrency_pipeline.mica`. For a comprehensive workflow that combines
concurrency, nested `using`, and auditing, explore
`examples/comprehensive_deployment.mica`.

## Generics and Bounds
```mica
fn max[T: Ord](a: T, b: T) -> T { if a < b { b } else { a } }
```
Recursive ADTs with traversal helpers appear in
`examples/generics_tree_algorithms.mica`.

## Collections and Loops
```mica
fn sum(xs: [Int]) -> Int { let mut s = 0; for x in xs { s = s + x }; s }
```

## Methods and Receivers
`impl` blocks introduce receiver-based methods that lower to simple HIR when you
run `--lower`.
```mica
impl Addable for Vec2 { fn add(self, other: Vec2) -> Vec2 { other } }
fn use_method(a: Vec2, b: Vec2) -> Vec2 { a.add(b) }
```

## CLI Shortcuts
- `--tokens`
- `--ast`
- `--ast --pretty`
- `--resolve`
- `--check`
- `--lower`

See `docs/snippets.md` for real CLI output.
