# Runtime and Capability Providers

> Capability-aware shims let generated binaries access host resources in a
> controlled, observable way.

## Overview

The runtime orchestrator wires capability-aware tasks to host-provided shims.
It executes them deterministically, records telemetry, and validates that
requested capabilities match what the compiler declared.

## Default Providers

`Runtime::with_default_shims()` registers console, time, filesystem, and
environment providers so examples and tests run without extra configuration.
Each provider mirrors the capability rows emitted by the compiler:

- **Console (`io`)** – `write_line` emits message events alongside unit results.
- **Time (`time`)** – `now_millis` returns the current wall-clock time.
- **Filesystem (`fs`)** – `read_to_string` and `write_string` transfer file
  contents while producing confirmation events.
- **Environment (`env`)** – `get`, `set`, and `unset` expose environment variable
  access with predictable clean-up semantics.

## Telemetry Surface

Every run produces a `RuntimeTrace` that contains ordered events, timestamps, and
per-task metrics (durations, capability counts, spawned tasks). Use
`RuntimeTrace::to_json_string()` or `Runtime::run_with_trace_json()` to export
telemetry for dashboards and tooling.

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
