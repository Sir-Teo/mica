# Compiler Pipeline Entry Points

[← Back to Module Reference](../module_reference.html) | [← Documentation Home](../index.html)

> The CLI exposes every stage of the compiler pipeline so tools can plug in
> without re-implementing compiler internals.

## Overview

`mica` surfaces a set of subcommands and flags that dump intermediate state from
lexing through code generation. The new `--pipeline-json` flag serialises a
module's journey through each stage, making it easy to inspect resolver output,
IR snapshots, and backend telemetry from a single invocation.

## CLI Cheatsheet

| Stage                  | Command snippet                                        | Output                           |
| ---------------------- | ------------------------------------------------------ | -------------------------------- |
| Tokens                 | `mica --tokens examples/hello.mica`                    | Stream of token kinds/lexemes    |
| AST                    | `mica --ast examples/hello.mica`                       | Pretty-printed syntax tree       |
| Resolver + Effects     | `mica --resolve-json examples/hello.mica`              | JSON graph of bindings/effects   |
| Pipeline summary       | `mica --pipeline-json examples/hello.mica`             | JSON object covering every stage |
| Runtime telemetry      | `mica --run --trace-json - examples/native_entry.mica` | Structured execution trace       |

Use the snippets as building blocks for editor tooling, CI auditing, or data
pipelines. The JSON outputs are stable, machine-readable structures that map
cleanly onto the documentation examples.

## Example Workflow

```bash
mica --pipeline-json examples/concurrency_pipeline.mica > pipeline.json
jq '.stages.resolve.effects' pipeline.json
mica --run --trace-json trace.json examples/native_entry.mica
jq '.summary.operation_counts' trace.json
```

The pipeline dump contains per-stage diagnostics, intermediate IR, and emitted
artifacts. Pair it with the runtime trace to correlate compile-time effects with
runtime behaviour.

### End-to-end walkthrough

1. **Capture the pipeline snapshot.** `mica --pipeline-json` emits resolver,
   checker, lowerer, and IR metrics in one JSON file. Use it to pre-populate IDE
   views or automation dashboards before any code executes.
2. **Execute with telemetry enabled.** `mica --run --trace-json trace.json` runs
   the native backend and writes a runtime trace that mirrors the in-process
   telemetry produced by the deterministic shims. Documentation snippets now
   capture this output automatically, keeping runtime telemetry under
   regression.
3. **Join the data sets.** The pipeline stages and runtime summary share the
   same capability and operation keys, so downstream tooling can correlate
   compile-time declarations with actual effects at runtime.
4. **Feed editor integrations.** Import both JSON files to power “peek
   capability usage” or “show runtime plan” commands without reverse-engineering
   internal compiler data structures.

## Integration Tips

1. **Cache intermediate results.** The JSON snapshot includes hashes for each
   stage, making it easy to determine when downstream tooling needs to refresh
   its caches.
2. **Compare stage deltas.** Store previous `--pipeline-json` outputs to diff
   resolver or IR changes across branches.
3. **Feed telemetry dashboards.** The runtime trace shares a schema with the
   pipeline dump, so observability tools can join compile-time and run-time
   perspectives.

## Next Steps

- Document the JSON schema formally so IDEs can generate typed bindings.
- Extend the walkthrough to correlate runtime trace snippets with the pipeline
  JSON so readers can follow effects from declaration to execution.
- Add worked examples to the language tour that reference the pipeline dumps.

---

## Related Modules

- **[CLI Tooling](cli.html)** — Command-line interface implementation
- **[Runtime](runtime.html)** — Telemetry generation
- **[SSA IR](ir.html)** — Intermediate representation exports

[← Back to Module Reference](../module_reference.html) | [← Documentation Home](../index.html)
