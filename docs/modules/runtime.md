# Runtime and Capability Providers

## Scope

The runtime orchestrator wires capability-aware tasks to concrete providers,
executes their plans in a deterministic FIFO order, and captures telemetry for
analysis or tooling consumption.【F:src/runtime/mod.rs†L1-L236】 This guide
summarizes the stock providers, telemetry surface, and quick-start examples for
embedding the runtime in host tools.

## Default providers

`Runtime::with_default_shims()` registers the console, time, filesystem, and
environment providers so language tour examples and tests run without manual
configuration.【F:src/runtime/mod.rs†L39-L64】 Each provider exposes a narrow API
that mirrors the effect rows emitted by the compiler:

- **Console (`io`)** – Supports `write_line`, emitting message events alongside a
  unit return value.【F:src/runtime/mod.rs†L874-L905】
- **Time (`time`)** – Provides `now_millis`, returning the current wall-clock time
  as an integer and emitting a data event.【F:src/runtime/mod.rs†L1074-L1096】
- **Filesystem (`fs`)** – Implements `read_to_string` and `write_string`. Reads
  surface file contents, while writes accept `"path=contents"` payloads and emit
  confirmation messages.【F:src/runtime/mod.rs†L909-L989】 Regression tests cover
  both read and write paths to guarantee the provider's behaviour.【F:src/tests/runtime_tests.rs†L190-L276】
- **Environment (`env`)** – Supports `get`, `set`, and `unset` for process
  environment variables, surfacing data and message events as appropriate.【F:src/runtime/mod.rs†L993-L1068】 Tests assert that the provider round-trips values and cleans up state after execution.【F:src/tests/runtime_tests.rs†L278-L324】

## Telemetry and JSON traces

Every task execution yields a `RuntimeTrace` containing ordered events, per-event
telemetry (sequence IDs and timestamps), and per-task metrics (durations,
capability counts, spawned task totals).【F:src/runtime/mod.rs†L96-L352】 The
trace can be serialized via `RuntimeTrace::to_json_string()` or produced directly
through `Runtime::run_with_trace_json()` for downstream tooling.【F:src/runtime/mod.rs†L136-L155】【F:src/runtime/mod.rs†L440-L489】 Tests ensure the
telemetry surface stays consistent and JSON encoding remains stable.【F:src/tests/runtime_tests.rs†L35-L188】【F:src/tests/runtime_tests.rs†L327-L384】

## Example

The snippet below demonstrates wiring the default providers, executing a task,
and consuming the telemetry report:

```rust
use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};

let runtime = Runtime::with_default_shims()?;
let spec = TaskSpec::new("main").with_capabilities(["io"]);
let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));

runtime.spawn(spec, plan);
let trace = runtime.run_with_telemetry()?;
assert_eq!(trace.events().len(), trace.telemetry().len());
assert_eq!(trace.tasks()[0].capability_counts.get("io"), Some(&1));
```

Use `RuntimeTrace::to_json_string()` if you need a serialized representation for
diagnostic dashboards or build artifacts.【F:src/runtime/mod.rs†L440-L489】
