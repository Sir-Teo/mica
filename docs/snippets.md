# CLI Snippets

This page shows short outputs from the CLI for selected examples.

## Pretty AST (`--ast --pretty`)

Command: `cargo run -- --ast --pretty examples/adt.mica`

```
module demo.adt
pub type Option[T] = Some(T) | None
pub type Result[T, E] = Ok(T) | Err(E)
pub fn map_option[T, U](x: Option[T], f: fn(T) -> U) -> Option[U] { … }
```

## Exhaustiveness Check (`--check`)

Command: `cargo run -- --check examples/adt_match_nonexhaustive.mica`

```
warning: non-exhaustive match for Color: missing variants Green, Blue
```

## Lowered HIR (`--lower`)

Command: `cargo run -- --lower examples/methods.mica`

```
hir module demo.methods
fn use_method(a, b)
  add(a, b)
```

Command: `cargo run -- --lower examples/spawn_await.mica`

```
hir module demo.concurrent
fn fetch(u, net) !{net}
  await(spawn(http::get(u, net)))
```

