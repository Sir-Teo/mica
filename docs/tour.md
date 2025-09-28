# Mica Tour (Prototype)

This is a quick, example-driven tour of Mica’s core surface.

## Modules

- One module per file: `module a.b.c`
- Public exports are marked with `pub`

## Types and ADTs

- Algebraic data types via `type`:

```mica
module demo.adt
pub type Option[T] = Some(T) | None
pub type Result[T, E] = Ok(T) | Err(E)
```

- Records are structural:

```mica
type Row = { id: Int, qty: Int, price: F64 }
```

## Pattern Matching

- Patterns over ADTs, tuples, and records; exhaustive by default.

```mica
module demo.matching

type Color = Red | Green | Blue

fn to_text(c: Color) -> String {
  match c { Red => "red", Green => "green", Blue => "blue" }
}
```

- Non-exhaustive matches are flagged (see `examples/adt_match_nonexhaustive.mica`).

## Effects and Using

- Effects are declared with effect rows; capabilities are explicit.

```mica
fn open_and_print(io: IO) !{io} {
  using File::open("/tmp/example.txt", io)? { io.println("opened ok") }
}

fn run_with(callback: fn(Int) -> Int !{io}, io: IO) !{io} {
  let _ = callback(42)
}
```

## Concurrency

- Structured tasks with `spawn` and `await`:

```mica
fn fetch(u: String, net: Net) -> Bytes !{net} {
  await spawn http::get(u, net)
}
```

## Generics and Bounds

- Generic functions with simple bounds:

```mica
fn max[T: Ord](a: T, b: T) -> T { if a < b { b } else { a } }
```

## Collections and Loops

- List type and `for` loops:

```mica
fn sum(xs: [Int]) -> Int { let mut s = 0; for x in xs { s = s + x }; s }
```

## Methods and Receivers

- `impl` blocks with `self` receivers; method calls lower to simple HIR (`--lower`).

```mica
impl Addable for Vec2 { fn add(self, other: Vec2) -> Vec2 { other } }
fn use_method(a: Vec2, b: Vec2) -> Vec2 { a.add(b) }
```

## CLI

- Tokens: `--tokens`
- AST: `--ast`
- Pretty AST: `--ast --pretty`
- Resolve ADTs: `--resolve`
- Exhaustiveness: `--check`
- Lower to HIR: `--lower`

See `docs/snippets.md` for example outputs.

