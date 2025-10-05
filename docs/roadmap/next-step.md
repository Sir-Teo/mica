# Immediate Next Step Recommendation

Phase 2 is complete: the typed IR, backend surface area, diagnostics, and
pureness analysis have shipped. Mica is now firmly executing the Phase 3
"Backend & Runtime" milestone, where the focus shifts to production-ready
runtimes, native code generation, and contributor tooling.

With that context, the next actions should reinforce Phase 3 outcomes:

1. **Broaden process orchestration.** Extend the scripted subprocess provider
   with error fixtures, argument validation, and tutorial coverage so complex
   workflows remain deterministic.
2. **Correlate trace walkthroughs.** Use the new CLI trace snippet to expand
   the walkthrough so readers can follow telemetry from compilation to
   execution without manual capture steps.
3. **Tune the parallel backend.** Use the worker/schedule metrics to profile
   workspace-sized builds and adjust heuristics before scaling out.
4. **Publish richer walkthroughs.** Expand the pipeline guide with editor-facing
   recipes that connect resolver/IR snapshots to runtime traces.

Staying disciplined on these items keeps Phase 3 deliverables aligned: richer
runtime coverage, observable compiler behaviour, and ergonomic tooling that can
underpin the subsequent IDE-focused Phase 4 work.
