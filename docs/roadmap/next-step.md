# Immediate Next Step Recommendation

Phase 2 is complete: the typed IR, backend surface area, diagnostics, and
pureness analysis have shipped. Mica is now firmly executing the Phase 3
"Backend & Runtime" milestone, where the focus shifts to production-ready
runtimes, native code generation, and contributor tooling.

With that context, the next actions should reinforce Phase 3 outcomes:

1. **Broaden deterministic capability providers.** Extend the runtime’s stock
   shims with filesystem coverage, deterministic network fixtures, and the
   accompanying smoke tests so compiled programs can exercise richer IO while
   preserving repeatability.
2. **Surface richer telemetry.** Emit aggregated runtime summaries (tasks,
   spawned children, capability usage) and JSON tooling snapshots so IDEs and
   continuous integration jobs can reason about executions without re-parsing
   logs.
3. **Track backend parallelism limits.** Capture worker concurrency and module
   scheduling metrics from the parallel backend harness to guide scaling
   experiments and capacity planning.
4. **Document the pipeline entry points.** Keep CLI references and developer
   notes aligned with the new `--pipeline-json` command and runtime fixtures so
   contributors can quickly reproduce the current system behaviour.

Staying disciplined on these items keeps Phase 3 deliverables aligned: richer
runtime coverage, observable compiler behaviour, and ergonomic tooling that can
underpin the subsequent IDE-focused Phase 4 work.
