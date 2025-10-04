# Syntax Front-End

> Lexing, parsing, and pretty-printing form the foundation for every other
> compiler stage.

## Overview

The syntax front-end lives in `src/syntax` and `src/pretty`. It turns UTF-8
source into an abstract syntax tree (AST) and is responsible for round-tripping
it back to readable code. This phase anchors Phase 0 of the roadmap and feeds
all later stages.

## Major Components

### Tokens
- Enumerate punctuation, keywords, and literal forms for the language.
- Serve as the single source of truth when evolving the syntax.

### Lexer
- Streams over UTF-8 input, emits spans for every token, and raises diagnostics
  for malformed literals or unexpected characters.
- Appends an explicit EOF marker so the parser can report friendly errors.

### Abstract Syntax Tree
- Represents modules, imports, capabilities, types, and expressions—including
  control flow, pattern matching, and effect annotations.
- Provides helpers (for example `Function::effect_row`) that semantic analysis
  reuses.

### Parser
- Hand-written recursive descent with targeted expectation helpers for actionable
  diagnostics.
- Handles precedence, effect annotations, and capability clauses so downstream
  passes do not need to interpret tokens directly.

### Pretty Printer
- Converts the AST back into formatted source for the CLI `--pretty` flag,
  documentation snippets, and regression tests.
- Covers the current language surface so round-tripping stays lossless.

## Integration Notes

1. Lexing and parsing reuse the shared diagnostics module so spans and messages
   reach the CLI unchanged.
2. The AST is consumed by semantic analysis, lowering, and IR generation without
   additional token inspection.
3. The pretty-printer powers documentation snapshots, keeping guides aligned with
   real compiler output.

## Roadmap Alignment

- **Phase 0** – Establishes precise spans and round-tripping as the baseline.
- **Phase 1** – Rich AST structures support upcoming inference and borrow
  checking work.
- **Future phases** – Lexer and parser helpers are intentionally modular to ease
  incremental parsing, doc comments, and other roadmap milestones.

## Next Steps

- Investigate incremental lexing and error recovery once watch-mode work begins.
- Expand tokens and AST nodes as new language constructs are approved.
- Add formatting toggles to the pretty-printer to satisfy tooling integrations.
