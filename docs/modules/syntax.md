# Syntax Front-End

## Scope

The syntax front-end spans lexing, token definitions, the abstract syntax tree,
parsing, and pretty-printing support housed under `src/syntax` and
`src/pretty`. It forms Phase 0 of the compiler roadmap and feeds every later
stage.

## Module Breakdown

### Tokens (`src/syntax/token.rs`)

- Enumerates lexical token kinds, punctuation, keywords, and literal holders for
  the language.【F:src/syntax/token.rs†L1-L165】
- Acts as the single source of truth when expanding the language’s surface
  syntax per the roadmap’s language evolution milestones.【F:docs/roadmap/compiler.md†L9-L74】

### Lexer (`src/syntax/lexer.rs`)

- Implements a streaming lexer that advances through the UTF-8 source while
  tracking byte indices for every token span.【F:src/syntax/lexer.rs†L1-L118】
- Produces numbers, strings, identifiers, comments, and structural punctuation,
  appending an explicit EOF sentinel for parser ergonomics.【F:src/syntax/lexer.rs†L106-L191】
- Emits `Error` diagnostics for malformed literals or unexpected characters,
  aligning with the roadmap’s emphasis on precise error reporting.【F:src/syntax/lexer.rs†L120-L191】【F:docs/roadmap/compiler.md†L20-L69】

### Abstract Syntax Tree (`src/syntax/ast.rs`)

- Defines rich enums and structs for modules, imports, type aliases, `impl`
  blocks, effect annotations, and expression forms such as control flow, pattern
  matching, and capability usage.【F:src/syntax/ast.rs†L1-L270】
- Provides accessor helpers like `Function::effect_row` that downstream passes
  reuse during semantic analysis.【F:src/syntax/ast.rs†L260-L332】
- Already models generics and capability clauses, preparing for roadmap phases
  that add inference and borrow checking.【F:docs/roadmap/compiler.md†L76-L125】

### Parser (`src/syntax/parser.rs`)

- Hand-written recursive-descent parser orchestrated through `Parser::parse`
  entry points for modules, items, expressions, and patterns.【F:src/syntax/parser.rs†L1-L205】
- Uses targeted expectation helpers to surface actionable diagnostics on missing
  tokens, fulfilling the roadmap’s lossless parsing goals.【F:src/syntax/parser.rs†L7-L44】【F:docs/roadmap/compiler.md†L39-L74】
- Handles expression precedence and effect annotations, enabling the semantic
  checker to reason about capability usage.【F:src/syntax/parser.rs†L125-L205】

### Pretty Printer (`src/pretty/mod.rs`)

- Converts the AST back into formatted source, supporting CLI `--pretty` output,
  documentation examples, and parser regression tests.【F:src/pretty/mod.rs†L1-L80】
- Covers type expressions, effect rows, and sum-type variants to guarantee
  round-tripping across the language features present today.【F:src/pretty/mod.rs†L21-L121】

## Integration Highlights

1. Lexing and parsing share the diagnostics infrastructure to propagate spans and
   messages consistently to the CLI and tests.【F:src/syntax/lexer.rs†L120-L191】【F:src/main.rs†L49-L74】
2. The parser produces AST nodes consumed by semantic analysis, lowering, and IR
   stages without needing to re-interpret tokens.【F:src/semantics/check.rs†L1-L217】【F:src/lower/mod.rs†L1-L200】
3. Pretty-printing is leveraged by the documentation snapshot generator to keep
   guides synchronized with the actual surface syntax.【F:src/bin/gen_snippets.rs†L1-L60】

## Roadmap Alignment

- **Phase 0:** Solidifies lexing, parsing, and pretty-printing capabilities with
  precise spans and round-tripping, matching the roadmap’s baseline goals.【F:docs/roadmap/compiler.md†L9-L74】
- **Phase 1:** The AST’s expressive type and effect modeling provides the raw
  data required for the semantic phases that introduce type inference and borrow
  checking.【F:docs/roadmap/compiler.md†L76-L125】
- **Phase 2+:** Lexer infrastructure is prepared for incremental updates and doc
  comments, while the parser’s modular helpers facilitate incremental and lossless
  parsing work later in the roadmap.【F:docs/roadmap/compiler.md†L39-L115】

## Next Steps

- Implement incremental lexing and parser error recovery once the roadmap reaches
  those milestones, ensuring CLI watch-modes remain responsive.
- Expand the token and AST catalogs as new language constructs are greenlit.
- Add formatting configuration hooks to the pretty printer to align with tooling
  integration plans.
