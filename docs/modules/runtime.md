# Runtime and Capability Providers

> Capability-aware shims let generated binaries access host resources in a
> controlled, observable way.

## Overview

The runtime orchestrator wires capability-aware tasks to host-provided shims.
It executes them deterministically, records telemetry, and validates that
requested capabilities match what the compiler declared.

## Default Providers

`Runtime::with_default_shims()` registers console, time, filesystem, network,
and environment providers so examples and tests run without extra configuration.
Each provider mirrors the capability rows emitted by the compiler:

- **Console (`io`)** – `write_line` emits message events alongside unit results.
- **Time (`time`)** – `now_millis` returns the current wall-clock time.
- **Filesystem (`fs`)** – `read_to_string` and `write_string` transfer file
  contents while producing confirmation events.
- **Environment (`env`)** – `get`, `set`, and `unset` expose environment
  variable access with predictable clean-up semantics.
- **Network (`net`)** – `fetch` resolves requests against pre-registered
  fixtures without hitting the live network.

### Deterministic Shims for Tests

The host-backed providers are convenient for running real binaries, but tests
often need reproducible behaviour. `Runtime::with_deterministic_shims()`
returns a bundle containing in-memory providers that mirror the same
capabilities without touching the host environment:

- **Deterministic console** captures `write_line` output and can script
  `read_line` responses.
- **In-memory filesystem** stores file contents in a hash map so tests can
  assert on writes and seed reads without touching disk.
- **Scripted environment** keeps key/value pairs in memory and exposes helper
  methods to seed fixtures.
- **Deterministic clock** returns scripted or monotonic timestamps, making it
  trivial to assert on runtime telemetry.

```rust
use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};

let bundle = Runtime::with_deterministic_shims()?;
let runtime = bundle.runtime();
bundle.console.queue_input("scripted");
bundle.time.push_time(42);

let spec = TaskSpec::new("main").with_capabilities(["io", "time"]);
let plan = TaskPlan::new()
    .invoke("io", "read_line", None)
    .invoke("time", "now_millis", None);

runtime.spawn(spec, plan);
runtime.run()?;
assert_eq!(bundle.time.last_emitted(), Some(43));
```

## Telemetry Surface

Every run produces a `RuntimeTrace` that contains ordered events, timestamps,
and per-task metrics. The telemetry now captures capability and operation
counts, as well as the time spent servicing each capability invocation. Use
`RuntimeTrace::to_json_string()` or `Runtime::run_with_trace_json()` to export
telemetry for dashboards and tooling.

The JSON summary section has the following shape:

```json
{
  "total_tasks": 2,
  "total_events": 8,
  "spawned_tasks": 1,
  "capability_counts": {"io": 2},
  "operation_counts": {"io::write_line": 2},
  "capability_durations_micros": {"io": 152},
  "operation_durations_micros": {"io::write_line": 152}
}
```

## Quick Start

```rust
use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};

let runtime = Runtime::with_default_shims()?;
let spec = TaskSpec::new("main").with_capabilities(["io"]);
let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));

runtime.spawn(spec, plan);
let trace = runtime.run_with_telemetry()?;
assert_eq!(trace.events().len(), trace.telemetry().len());
```

Serialise the trace when you need to persist telemetry or feed observability
pipelines. The stable JSON structure makes it easy to plug into custom tools.
Pair the deterministic shims with the JSON output to snapshot runtime behaviour
in tests and continuous integration.
