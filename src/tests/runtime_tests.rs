use crate::runtime::{
    CapabilityEvent, Runtime, RuntimeErrorKind, RuntimeEvent, RuntimeValue, TaskPlan, TaskSpec,
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
    let valid = TaskSpec::new("main").with_capabilities(["io"]);
    runtime
        .ensure_capabilities(&valid)
        .expect("io capability available");

    let missing = TaskSpec::new("main").with_capabilities(["net"]);
    let err = runtime
        .ensure_capabilities(&missing)
        .expect_err("expected missing capability error");
    assert!(matches!(
        err.kind(),
        RuntimeErrorKind::UnknownCapability { name } if name == "net"
    ));
}
