# Diagnostics Infrastructure

> A single diagnostics vocabulary keeps user-facing errors consistent across all
> compiler stages.

## Overview

The `src/diagnostics` module defines error types and helpers that every stage of
the compiler reuses. By centralising formatting, span tracking, and result
aliases, the CLI, tests, and future IDE integrations can rely on the same output
structure.

## Responsibilities

- **Error representation** – `Error` and `ErrorKind` capture the category,
  optional span, and user-facing message for compiler failures.
- **Helpers** – Smart constructors (`Error::lex`, `Error::parse`, etc.) reduce
  boilerplate at call sites and encourage consistent span usage.
- **Display formatting** – Implementations render friendly messages that include
  span ranges when available, powering both CLI output and snapshot tests.
- **Prelude exports** – `mod.rs` re-exports the common types and a shared
  `Result<T>` alias so each stage uses the same API surface.

## Integration Notes

- Lexing and parsing return the shared `Result<T>` alias, letting diagnostics
  bubble up naturally into the CLI.
- The crate re-exports diagnostics via `mica::error`, enabling downstream
  consumers to hook into identical types when embedding the compiler.

## Roadmap Alignment

- **Immediate** – Span-aware errors satisfy the prototype’s requirement for
  actionable CLI diagnostics.
- **Near future** – Semantic and backend passes will add new error variants; the
  module is designed to evolve without breaking callers.
- **Tooling** – Planned IDE integrations require machine-readable diagnostics,
  so serialisation hooks and diagnostic codes are natural follow-ups.

## Next Steps

1. Prototype structured payloads (labels, notes, hints) that keep CLI output
   readable while giving IDEs rich data.
2. Introduce conversion traits for upcoming semantic error types to preserve a
   unified surface area.
3. Explore warning levels and categories that downstream tools can filter or
   promote to hard errors.
