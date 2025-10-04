# Immediate Next Step Recommendation

The Phase 1 semantics milestone is complete: the resolver now builds cross-module
workspaces with capability metadata, and the checker validates signatures,
effects, and pattern exhaustiveness through `mica --check`.

With that foundation in place, focus on the early Phase 2 deliverables:

1. **Define the typed IR contract.** Finalise the SSA-inspired layout, encode
effect metadata, and begin lowering typed ASTs so later passes can reason about
drops and scheduling.
2. **Sketch backend integration.** Outline the LLVM-oriented backend scaffolding
that will consume the IR, ensuring the lowering data model preserves sufficient
provenance for code generation and runtime hooks.
3. **Expand the diagnostics playbook.** Capture the semantics of the new checker
errors, refresh CLI examples, and document wording expectations so tooling work
can consume consistent output.

Completing these items transitions the project into Phase 2 and sets up the SSA
lowering, backend, and optimisation research that follow.

_Status update:_ The typed IR guide now lives in `docs/modules/ir.md`, the LLVM
scaffolding backend exports its contract via `mica --llvm`, and the CLI snippets
showcase the richer pipeline so contributors can iterate confidently.
