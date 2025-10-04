# Immediate Next Step Recommendation

Phase 2 is complete: the typed IR, backend surface area, diagnostics, and
pureness analysis have shipped. Mica is now firmly executing the Phase 3
"Backend & Runtime" milestone, where the focus shifts to production-ready
runtimes, native code generation, and contributor tooling.

With that context, the next actions should reinforce Phase 3 outcomes:

1. **Add process orchestration shims.** Layer scripted subprocess/task providers
   on top of the deterministic runtime bundle so multi-process programs stay
   reproducible.
2. **Export telemetry from the CLI.** Extend `mica run` with opt-in trace output
   flags and document how downstream tooling can ingest the JSON metrics.
3. **Tune the parallel backend.** Use the new worker/schedule metrics to profile
   workspace-sized builds and adjust heuristics before scaling out.
4. **Publish pipeline walkthroughs.** Turn the documented pipeline entry points
   into step-by-step guides that show resolver/IR snapshots flowing into editor
   integrations and automation.

Staying disciplined on these items keeps Phase 3 deliverables aligned: richer
runtime coverage, observable compiler behaviour, and ergonomic tooling that can
underpin the subsequent IDE-focused Phase 4 work.
