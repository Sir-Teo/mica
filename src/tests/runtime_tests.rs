use crate::runtime::{
    CapabilityEvent, CompletedProcess, NetworkFixture, Runtime, RuntimeErrorKind, RuntimeEvent,
    RuntimeValue, ScriptedProcess, TaskPlan, TaskSpec, register_network_fixture,
    reset_network_fixtures,
};

#[test]
fn runtime_executes_default_shims() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["io", "time"]);
    let plan = TaskPlan::new()
        .invoke("io", "write_line", Some(RuntimeValue::from("hello")))
        .invoke("time", "now_millis", None);

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");
    assert!(matches!(events[0], RuntimeEvent::TaskStarted { ref task } if task == "main"));
    assert!(matches!(
        events[1],
        RuntimeEvent::CapabilityInvoked {
            ref task,
            ref capability,
            ref operation
        } if task == "main" && capability == "io" && operation == "write_line"
    ));
    assert!(matches!(
        events[2],
        RuntimeEvent::CapabilityEvent {
            ref task,
            ref capability,
            event: CapabilityEvent::Message(ref msg)
        } if task == "main" && capability == "io" && msg == "hello"
    ));
    assert!(matches!(
        events[3],
        RuntimeEvent::CapabilityInvoked {
            ref task,
            ref capability,
            ref operation
        } if task == "main" && capability == "time" && operation == "now_millis"
    ));
    assert!(matches!(
        events[4],
        RuntimeEvent::CapabilityEvent {
            ref task,
            ref capability,
            event: CapabilityEvent::Data(RuntimeValue::Int(_))
        } if task == "main" && capability == "time"
    ));
    assert!(matches!(events[5], RuntimeEvent::TaskCompleted { ref task } if task == "main"));
}

#[test]
fn runtime_enforces_capability_scopes() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["time"]);
    let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("nope")));

    runtime.spawn(spec, plan);
    let err = runtime
        .run()
        .expect_err("expected missing capability error");
    assert!(matches!(
        err.kind(),
        RuntimeErrorKind::MissingCapability {
            task,
            capability
        } if task == "main" && capability == "io"
    ));
}

#[test]
fn runtime_schedules_child_tasks_fifo() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let child_spec = TaskSpec::new("child").with_capabilities(["io"]);
    let child_plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("child")));
    let parent_plan = TaskPlan::new()
        .invoke("io", "write_line", Some(RuntimeValue::from("parent")))
        .spawn(child_spec.clone(), child_plan);
    let parent_spec = TaskSpec::new("parent").with_capabilities(["io"]);

    runtime.spawn(parent_spec, parent_plan);
    let events = runtime.run().expect("runtime events");

    let expected_order = [
        RuntimeEvent::TaskStarted {
            task: "parent".into(),
        },
        RuntimeEvent::CapabilityInvoked {
            task: "parent".into(),
            capability: "io".into(),
            operation: "write_line".into(),
        },
        RuntimeEvent::CapabilityEvent {
            task: "parent".into(),
            capability: "io".into(),
            event: CapabilityEvent::Message("parent".into()),
        },
        RuntimeEvent::TaskScheduled {
            parent: "parent".into(),
            child: "child".into(),
        },
        RuntimeEvent::TaskCompleted {
            task: "parent".into(),
        },
        RuntimeEvent::TaskStarted {
            task: "child".into(),
        },
        RuntimeEvent::CapabilityInvoked {
            task: "child".into(),
            capability: "io".into(),
            operation: "write_line".into(),
        },
        RuntimeEvent::CapabilityEvent {
            task: "child".into(),
            capability: "io".into(),
            event: CapabilityEvent::Message("child".into()),
        },
        RuntimeEvent::TaskCompleted {
            task: "child".into(),
        },
    ];

    assert_eq!(events, expected_order);
}

#[test]
fn runtime_validates_registered_capabilities() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let valid = TaskSpec::new("main").with_capabilities(["io", "process"]);
    runtime
        .ensure_capabilities(&valid)
        .expect("io capability available");

    let missing = TaskSpec::new("main").with_capabilities(["db"]);
    let err = runtime
        .ensure_capabilities(&missing)
        .expect_err("expected missing capability error");
    assert!(matches!(
        err.kind(),
        RuntimeErrorKind::UnknownCapability { name } if name == "db"
    ));
}

#[test]
fn deterministic_process_provider_replays_scripted_commands() {
    let bundle = Runtime::with_deterministic_shims().expect("runtime setup");
    bundle
        .process
        .script(ScriptedProcess::new("scripted-task").with_stdout_line("ok"));

    let spec = TaskSpec::new("main").with_capabilities(["process"]);
    let plan = TaskPlan::new().invoke(
        "process",
        "spawn",
        Some(RuntimeValue::from("scripted-task")),
    );

    bundle.runtime().spawn(spec, plan);
    let events = bundle.run().expect("runtime events");

    assert!(events.iter().any(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Message(message),
            ..
        } => capability == "process" && message.contains("stdout: ok"),
        _ => false,
    }));

    let completed: Vec<CompletedProcess> = bundle.process.completed();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].command, "scripted-task");
    assert_eq!(completed[0].exit_code, 0);
    assert_eq!(completed[0].stdout, vec!["ok".to_string()]);
}

#[cfg(unix)]
#[test]
fn process_provider_executes_host_commands() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["process"]);
    let plan = TaskPlan::new().invoke(
        "process",
        "spawn",
        Some(RuntimeValue::from("/bin/echo process-runtime")),
    );

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    assert!(events.iter().any(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Message(message),
            ..
        } => capability == "process" && message.contains("stdout: process-runtime"),
        _ => false,
    }));
}

#[cfg(unix)]
#[test]
fn process_provider_captures_blank_stdout_lines() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["process"]);
    let plan = TaskPlan::new().invoke("process", "spawn", Some(RuntimeValue::from("/bin/echo")));

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    assert!(events.iter().any(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Message(message),
            ..
        } => capability == "process" && message == "stdout: ",
        _ => false,
    }));
}

#[test]
fn runtime_produces_structured_telemetry() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["io"]);
    let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));

    runtime.spawn(spec, plan);
    let trace = runtime
        .run_with_telemetry()
        .expect("runtime telemetry trace");

    let events = trace.events();
    assert_eq!(events.len(), 4, "expected task lifecycle events");

    let telemetry = trace.telemetry();
    assert_eq!(telemetry.len(), events.len());

    for (index, (event, telemetry_entry)) in events.iter().zip(telemetry.iter()).enumerate() {
        assert_eq!(telemetry_entry.sequence, index);
        assert_eq!(&telemetry_entry.event, event);
        assert!(
            telemetry_entry.timestamp_micros.is_some(),
            "telemetry entry should include a timestamp"
        );
    }

    assert!(
        telemetry
            .windows(2)
            .all(|pair| pair[0].sequence + 1 == pair[1].sequence)
    );

    let tasks = trace.tasks();
    assert_eq!(tasks.len(), 1, "expected exactly one task metric entry");
    let metrics = &tasks[0];
    assert_eq!(metrics.task, "main");
    assert!(
        metrics.event_count >= events.len(),
        "task metrics should count at least as many events as observed"
    );
    assert!(
        metrics.start_timestamp_micros.is_some(),
        "task metrics should include a start timestamp"
    );
    assert_eq!(metrics.capability_counts.get("io"), Some(&1));
    assert_eq!(metrics.operation_counts.get("io::write_line"), Some(&1));
    assert!(
        metrics.capability_durations_micros.contains_key("io"),
        "expected capability duration entry for io"
    );
    assert!(
        metrics
            .operation_durations_micros
            .contains_key("io::write_line"),
        "expected operation duration entry for io::write_line"
    );
}

#[test]
fn runtime_filesystem_provider_reads_files() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("reader").with_capabilities(["fs"]);

    let temp_dir = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = temp_dir.join(format!("mica_runtime_test_{unique}.txt"));
    fs::write(&path, "filesystem contents").expect("write fixture");

    let plan = TaskPlan::new().invoke(
        "fs",
        "read_to_string",
        Some(RuntimeValue::from(path.to_string_lossy().to_string())),
    );

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");
    fs::remove_file(&path).ok();

    let observed = events.iter().find_map(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Data(RuntimeValue::String(data)),
            ..
        } if capability == "fs" => Some(data.clone()),
        _ => None,
    });

    assert_eq!(
        observed.as_deref(),
        Some("filesystem contents"),
        "filesystem provider should surface file contents"
    );
}

#[test]
fn runtime_filesystem_provider_writes_files() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("writer").with_capabilities(["fs"]);

    let temp_dir = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = temp_dir.join(format!("mica_runtime_write_{unique}.txt"));
    let path_string = path.to_string_lossy().to_string();
    let payload = format!("{}=runtime data", path_string);

    let plan = TaskPlan::new()
        .invoke("fs", "write_string", Some(RuntimeValue::from(payload)))
        .invoke(
            "fs",
            "read_to_string",
            Some(RuntimeValue::from(path_string.clone())),
        );

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    let observed = events.iter().find_map(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Data(RuntimeValue::String(data)),
            ..
        } if capability == "fs" => Some(data.clone()),
        _ => None,
    });

    assert_eq!(
        observed.as_deref(),
        Some("runtime data"),
        "filesystem provider should persist written data",
    );

    fs::remove_file(&path).ok();
}

#[test]
fn runtime_environment_provider_round_trips_values() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("env").with_capabilities(["env"]);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let key = format!("MICA_RUNTIME_ENV_{unique}");

    let plan = TaskPlan::new()
        .invoke(
            "env",
            "set",
            Some(RuntimeValue::from(format!("{key}=runtime"))),
        )
        .invoke("env", "get", Some(RuntimeValue::from(key.clone())))
        .invoke("env", "unset", Some(RuntimeValue::from(key.clone())));

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    let observed = events.iter().find_map(|event| match event {
        RuntimeEvent::CapabilityEvent {
            capability,
            event: CapabilityEvent::Data(RuntimeValue::String(data)),
            ..
        } if capability == "env" => Some(data.clone()),
        _ => None,
    });

    assert_eq!(
        observed.as_deref(),
        Some("runtime"),
        "environment provider should surface stored values",
    );

    assert!(
        std::env::var(&key).is_err(),
        "environment variable should be unset after runtime execution",
    );

    unsafe {
        std::env::remove_var(&key);
    }
}

#[test]
fn runtime_trace_reports_task_metrics_for_children() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let child_spec = TaskSpec::new("child").with_capabilities(["io"]);
    let child_plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("child")));
    let parent_plan = TaskPlan::new()
        .invoke("io", "write_line", Some(RuntimeValue::from("parent")))
        .spawn(child_spec.clone(), child_plan);
    let parent_spec = TaskSpec::new("parent").with_capabilities(["io"]);

    runtime.spawn(parent_spec, parent_plan);
    let trace = runtime
        .run_with_telemetry()
        .expect("runtime telemetry trace");

    let tasks = trace.tasks();
    assert_eq!(
        tasks.len(),
        2,
        "expected metrics for parent and child tasks"
    );

    let parent_metrics = tasks
        .iter()
        .find(|metrics| metrics.task == "parent")
        .expect("missing parent metrics");
    assert_eq!(parent_metrics.spawned_tasks, 1);
    assert_eq!(parent_metrics.capability_counts.get("io"), Some(&1));

    let child_metrics = tasks
        .iter()
        .find(|metrics| metrics.task == "child")
        .expect("missing child metrics");
    assert_eq!(child_metrics.spawned_tasks, 0);
    assert_eq!(child_metrics.capability_counts.get("io"), Some(&1));
}

#[test]
fn runtime_trace_serializes_to_json() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("main").with_capabilities(["io"]);
    let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));

    runtime.spawn(spec, plan);
    let json = runtime
        .run_with_trace_json()
        .expect("runtime should serialize trace");

    assert!(json.trim_start().starts_with('{'));
    assert!(json.trim_end().ends_with('}'));
    assert!(json.contains("\"events\""));
    assert!(json.contains("\"telemetry\""));
    assert!(json.contains("\"tasks\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("\"task\":\"main\""));
    assert!(json.contains("\"type\":\"task_started\""));
    assert!(json.contains("\"capability_counts\":{\"io\":1"));
    assert!(json.contains("\"operation_counts\""));
    assert!(json.contains("\"capability_durations_micros\""));
    assert!(json.contains("\"operation_durations_micros\""));
    assert!(json.contains("\"total_events\""));
}

#[test]
fn runtime_trace_summary_accumulates_metrics() {
    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let child_spec = TaskSpec::new("child").with_capabilities(["io"]);
    let child_plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("child")));
    let parent_plan = TaskPlan::new()
        .invoke("io", "write_line", Some(RuntimeValue::from("parent")))
        .spawn(child_spec.clone(), child_plan);
    let parent_spec = TaskSpec::new("parent").with_capabilities(["io"]);

    runtime.spawn(parent_spec, parent_plan);
    let trace = runtime
        .run_with_telemetry()
        .expect("runtime telemetry trace");

    let summary = trace.summary();
    assert_eq!(summary.total_tasks, 2);
    assert_eq!(summary.total_events, trace.events().len());
    assert_eq!(summary.spawned_tasks, 1);
    assert_eq!(summary.capability_counts.get("io"), Some(&2));
    assert_eq!(summary.operation_counts.get("io::write_line"), Some(&2));
    assert!(
        summary.capability_durations_micros.contains_key("io"),
        "expected aggregated io duration",
    );
    assert!(
        summary
            .operation_durations_micros
            .contains_key("io::write_line"),
        "expected aggregated io::write_line duration",
    );
}

#[test]
fn runtime_deterministic_shims_capture_state() {
    let bundle = Runtime::with_deterministic_shims().expect("runtime setup");
    let runtime = bundle.runtime();

    bundle.console.clear_writes();
    bundle.console.queue_input("scripted input");
    bundle.time.push_time(42);

    let spec = TaskSpec::new("main").with_capabilities(["io", "fs", "env", "time"]);
    let plan = TaskPlan::new()
        .invoke("io", "write_line", Some(RuntimeValue::from("hi")))
        .invoke("io", "read_line", None)
        .invoke(
            "fs",
            "write_string",
            Some(RuntimeValue::from("deterministic.txt=payload")),
        )
        .invoke(
            "fs",
            "read_to_string",
            Some(RuntimeValue::from("deterministic.txt")),
        )
        .invoke("env", "set", Some(RuntimeValue::from("TEMP=deterministic")))
        .invoke("env", "get", Some(RuntimeValue::from("TEMP")))
        .invoke("time", "now_millis", None)
        .invoke("time", "now_millis", None);

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    assert_eq!(bundle.console.writes(), vec!["hi".to_string()]);
    assert_eq!(
        bundle.filesystem.read_file("deterministic.txt"),
        Some("payload".to_string())
    );
    assert_eq!(bundle.env.get("TEMP"), Some("deterministic".to_string()));
    assert_eq!(bundle.time.last_emitted(), Some(43));

    let mut seen_time_values = Vec::new();
    let mut seen_input = false;
    for event in events {
        if let RuntimeEvent::CapabilityEvent {
            capability, event, ..
        } = event
        {
            match (capability.as_str(), event) {
                ("time", CapabilityEvent::Data(RuntimeValue::Int(value))) => {
                    seen_time_values.push(value);
                }
                ("io", CapabilityEvent::Data(RuntimeValue::String(value))) => {
                    if value == "scripted input" {
                        seen_input = true;
                    }
                }
                _ => {}
            }
        }
    }

    assert_eq!(seen_time_values, vec![42, 43]);
    assert!(seen_input, "expected scripted input to be surfaced");
}

#[test]
fn runtime_network_provider_serves_registered_fixtures() {
    reset_network_fixtures();
    register_network_fixture(
        "example",
        NetworkFixture::new(200, "payload").with_header("content-type", "text/plain"),
    );

    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("net").with_capabilities(["net"]);
    let plan = TaskPlan::new().invoke("net", "fetch", Some(RuntimeValue::from("example")));

    runtime.spawn(spec, plan);
    let events = runtime.run().expect("runtime events");

    assert!(events.iter().any(|event| {
        matches!(
            event,
            RuntimeEvent::CapabilityEvent {
                capability,
                event: CapabilityEvent::Data(RuntimeValue::Int(status)),
                ..
            } if capability == "net" && *status == 200
        )
    }));

    assert!(events.iter().any(|event| {
        matches!(
            event,
            RuntimeEvent::CapabilityEvent {
                capability,
                event: CapabilityEvent::Message(message),
                ..
            } if capability == "net" && message.contains("content-type: text/plain")
        )
    }));
}

#[test]
fn runtime_network_provider_reports_missing_fixtures() {
    reset_network_fixtures();

    let runtime = Runtime::with_default_shims().expect("runtime setup");
    let spec = TaskSpec::new("net").with_capabilities(["net"]);
    let plan = TaskPlan::new().invoke("net", "fetch", Some(RuntimeValue::from("missing")));

    runtime.spawn(spec, plan);
    let err = runtime.run().expect_err("expected missing fixture error");

    assert!(matches!(
        err.kind(),
        RuntimeErrorKind::ProviderFailure { capability, .. } if capability == "net"
    ));
}
