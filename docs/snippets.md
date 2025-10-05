# CLI Snippets

[← Back to Documentation Home](index.html)

This page shows short outputs from the CLI for selected examples. These snippets are automatically generated from real compiler output and verified in CI.

## Pretty AST (`--ast --pretty`)

Command: `cargo run --bin mica -- --ast --pretty examples/adt.mica`

```
module demo.adt
pub type Option[T] = Some(T) | None
pub type Result[T, E] = Ok(T) | Err(E)
pub fn map_option[T, U](x: Option[T], f: fn(T) -> U) -> Option[U] { … }
```

## Exhaustiveness Check (`--check`)

Command: `cargo run --bin mica -- --check examples/adt_match_nonexhaustive.mica`

```
warning: non-exhaustive match for Color: missing variants Green, Blue
```

## Lowered HIR (`--lower`)

Command: `cargo run --bin mica -- --lower examples/methods.mica`

```
hir module demo.methods
type Vec2 = Record([("x", Name("Int")), ("y", Name("Int"))])
fn use_method(a, b)
  add(a, b)
```

## Typed IR (`--ir`)

Command: `cargo run --bin mica -- --ir examples/methods.mica`

```
module demo.methods

fn use_method(a: Vec2, b: Vec2) -> Vec2
  block 0:
    %2 = call add(%0, %1) : _
    return %2
```

## LLVM Scaffold (`--llvm`)

Command: `cargo run --bin mica -- --llvm examples/methods.mica`

```
; ModuleID = 'demo.methods'
target datalayout = "e-m:e-p:64:64-i64:64-f64:64-n8:16:32:64-S128"

%record.Vec2 = type { i64, i64 }
; layout: size=16, align=8

define %record.Vec2 @use_method(%record.Vec2 %a, %record.Vec2 %b) {
bb0:
  ; block purity: effectful
  %2 = call ptr @add(%record.Vec2 %0, %record.Vec2 %1)
  ret ptr %2
}
```

Command: `cargo run --bin mica -- --lower examples/spawn_await.mica`

```
hir module demo.concurrent
func fetch(u, net) !{net}
  await(spawn(http::get(u, net)))
```

---

## Related Documentation

- **[Language Tour](tour.html)** — Learn the language syntax and semantics
- **[CLI Reference](modules/cli.html)** — Detailed CLI documentation
- **[Pipeline Entry Points](modules/pipeline.html)** — JSON exports and tooling integration

[← Back to Documentation Home](index.html)
