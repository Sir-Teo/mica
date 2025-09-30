# Diagnostics Infrastructure

## Scope

The diagnostics module (`src/diagnostics`) defines the error vocabulary shared
across lexing, parsing, and future compiler passes. It provides a consistent
reporting surface for the CLI, tests, and IDE integrations envisioned in the
roadmap.

## Current Responsibilities

| Component | Description |
| --- | --- |
| `Error` struct | Captures the error kind, optional byte-span, and human-readable message for any compiler failure.【F:src/diagnostics/error.rs†L5-L49】 |
| `ErrorKind` enum | Distinguishes lexical and parse errors today, with room to grow into semantic and backend categories as more passes emit diagnostics.【F:src/diagnostics/error.rs†L12-L33】【F:docs/roadmap/compiler.md†L20-L69】 |
| Smart constructors | `Error::lex` and `Error::parse` ensure call sites consistently attach spans and messages without reimplementing boilerplate.【F:src/diagnostics/error.rs†L18-L33】 |
| Display impls | Formatted output automatically includes span ranges when available, supporting both human-friendly CLI output and snapshot tests.【F:src/diagnostics/error.rs†L38-L58】 |
| Prelude exports | `src/diagnostics/mod.rs` re-exports `Error`, `ErrorKind`, and `Result`, so every stage shares a common diagnostic API.【F:src/diagnostics/mod.rs†L1-L3】 |

## Integration Points

- Lexing and parsing APIs return the shared `Result<T>` alias, guaranteeing that
  diagnostics flow upward into the CLI and tests without wrapping.【F:src/diagnostics/error.rs†L3-L4】【F:src/main.rs†L17-L74】
- `mica::error` is re-exported through `src/lib.rs`, allowing downstream crates
  to hook into the same error types once the compiler becomes an embeddable
  library.【F:src/lib.rs†L1-L13】【F:docs/roadmap/ecosystem.md†L22-L78】

## Roadmap Alignment

- **Phase 0:** The roadmap calls for richer spans and structured diagnostics; the
  existing span-aware error struct is designed to extend with labels and notes
  without breaking the API.【F:docs/roadmap/compiler.md†L20-L69】
- **Phase 1:** Type-checker and resolver diagnostics will add new variants—plan
  to grow `ErrorKind` or layer additional enums while keeping the shared `Error`
  facade for CLI compatibility.【F:docs/roadmap/compiler.md†L76-L125】
- **Phase 3:** IDE and tooling milestones require machine-readable diagnostics;
  consider augmenting the module with codes and JSON serialization to feed the
  planned language server.【F:docs/roadmap/tooling.md†L1-L60】

## Next Steps

1. Prototype structured diagnostic payloads (labels, notes, hints) to meet the
   roadmap’s IDE quality bar.
2. Add conversion traits from upcoming semantic error types so the entire
   pipeline remains unified.
3. Introduce feature flags or categories that downstream tools can filter by,
   paving the way for warning levels and lint passes.

## Phase 2 Diagnostics Playbook

1. **Codify happy-path baselines.** The pipeline regression suite exercises the
   parser, pretty-printer, lowerer, resolver, and checker together, ensuring
   “clean” modules stay noise-free as features expand.【F:src/tests/pipeline_tests.rs†L1-L139】
2. **Enforce negative coverage.** Semantic regression tests purposely miss
   variants and capability bindings so the checker continues to surface the
   right diagnostics with actionable messaging.【F:src/tests/pipeline_tests.rs†L106-L139】【F:src/tests/resolve_and_check_tests.rs†L128-L205】
3. **Track roadmap alignment.** Each milestone documents diagnostic exit
   criteria so we know when Phase 2’s richer IR and backend hooks are ready to
   surface structured errors downstream.【F:docs/roadmap/milestones.md†L1-L120】
4. **Snapshot backend surfacing.** The backend regression suite now asserts
   that both textual and LLVM scaffolding outputs retain capability metadata,
   giving diagnostics parity between CLI previews and future native builds.【F:src/tests/backend_tests.rs†L1-L96】
