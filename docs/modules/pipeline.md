# Compiler Pipeline Entry Points

> The CLI exposes every stage of the compiler pipeline so tools can plug in
> without re-implementing compiler internals.

## Overview

`mica` surfaces a set of subcommands and flags that dump intermediate state from
lexing through code generation. The new `--pipeline-json` flag serialises a
module's journey through each stage, making it easy to inspect resolver output,
IR snapshots, and backend telemetry from a single invocation.

## CLI Cheatsheet

| Stage                  | Command snippet                               | Output                      |
| ---------------------- | --------------------------------------------- | --------------------------- |
| Tokens                 | `mica --tokens examples/hello.mica`           | Stream of token kinds/lexemes |
| AST                    | `mica --ast examples/hello.mica`              | Pretty-printed syntax tree  |
| Resolver + Effects     | `mica --resolve-json examples/hello.mica`     | JSON graph of bindings/effects |
| Pipeline summary       | `mica --pipeline-json examples/hello.mica`    | JSON object covering every stage |
| Runtime telemetry      | `mica run examples/hello.mica --trace-json -` | Structured execution trace  |

Use the snippets as building blocks for editor tooling, CI auditing, or data
pipelines. The JSON outputs are stable, machine-readable structures that map
cleanly onto the documentation examples.

## Example Workflow

```bash
mica --pipeline-json examples/concurrency_pipeline.mica > pipeline.json
jq '.stages.resolve.effects' pipeline.json
mica run examples/concurrency_pipeline.mica --trace-json trace.json
jq '.summary.operation_counts' trace.json
```

The pipeline dump contains per-stage diagnostics, intermediate IR, and emitted
artifacts. Pair it with the runtime trace to correlate compile-time effects with
runtime behaviour.

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
- Extend `mica run` with `--trace-json <path>` to persist telemetry without
  piping stdout.
- Add worked examples to the language tour that reference the pipeline dumps.
