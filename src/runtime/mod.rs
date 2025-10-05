use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::backend::BackendError;

/// Runtime orchestrator that maps declared capability requirements to concrete
/// providers and executes task plans in a deterministic FIFO order.
#[derive(Debug, Clone, Default)]
pub struct Runtime {
    inner: Arc<RuntimeInner>,
}

/// Convenience bundle returned by [`Runtime::with_deterministic_shims`] that
/// exposes the deterministic providers alongside the configured runtime so
/// tests can observe captured state.
#[derive(Debug, Clone)]
pub struct DeterministicRuntime {
    runtime: Runtime,
    pub console: DeterministicConsoleProvider,
    pub filesystem: InMemoryFilesystemProvider,
    pub env: DeterministicEnvProvider,
    pub time: DeterministicTimeProvider,
    pub process: DeterministicProcessProvider,
}

impl std::ops::Deref for DeterministicRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl DeterministicRuntime {
    /// Returns a clone of the configured runtime for ergonomic chaining in
    /// tests without exposing the internal Arc directly.
    pub fn runtime(&self) -> Runtime {
        self.runtime.clone()
    }

    /// Consumes the bundle and returns the owned runtime.
    pub fn into_runtime(self) -> Runtime {
        self.runtime
    }
}

#[derive(Debug, Default)]
struct RuntimeInner {
    providers: RwLock<HashMap<String, Arc<dyn CapabilityProvider>>>,
    scheduler: Scheduler,
}

#[derive(Debug, Default)]
struct Scheduler {
    queue: Mutex<VecDeque<TaskEntry>>,
}

#[derive(Debug, Clone)]
struct TaskEntry {
    spec: TaskSpec,
    plan: TaskPlan,
}

impl Runtime {
    /// Creates an empty runtime without any registered providers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the stock providers used in the language tour (console IO,
    /// wall-clock time, filesystem, and environment access) so capability-driven
    /// examples can execute out of the box.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mica::runtime::{Runtime, RuntimeEvent, RuntimeValue, TaskPlan, TaskSpec};
    /// let runtime = Runtime::with_default_shims().expect("runtime setup");
    /// let spec = TaskSpec::new("main").with_capabilities(["io"]);
    /// let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));
    /// runtime.spawn(spec, plan);
    /// let events = runtime.run().expect("runtime events");
    /// assert!(matches!(
    ///     events.last(),
    ///     Some(RuntimeEvent::TaskCompleted { task }) if task == "main"
    /// ));
    /// ```
    pub fn with_default_shims() -> Result<Self, RuntimeError> {
        let runtime = Runtime::new();
        runtime.register_provider(ConsoleProvider)?;
        runtime.register_provider(TimeProvider)?;
        runtime.register_provider(FilesystemProvider)?;
        runtime.register_provider(NetworkProvider)?;
        runtime.register_provider(EnvProvider)?;
        runtime.register_provider(ProcessProvider::default())?;
        Ok(runtime)
    }

    /// Registers deterministic, in-memory capability providers that are safe to
    /// use in tests. The returned bundle includes handles to each provider so
    /// callers can seed fixtures or assert on captured state.
    pub fn with_deterministic_shims() -> Result<DeterministicRuntime, RuntimeError> {
        let runtime = Runtime::new();
        let console = DeterministicConsoleProvider::default();
        let filesystem = InMemoryFilesystemProvider::default();
        let env = DeterministicEnvProvider::default();
        let time = DeterministicTimeProvider::monotonic(0, 1);
        let process = DeterministicProcessProvider::default();

        runtime.register_provider(console.clone())?;
        runtime.register_provider(time.clone())?;
        runtime.register_provider(filesystem.clone())?;
        runtime.register_provider(NetworkProvider)?;
        runtime.register_provider(env.clone())?;
        runtime.register_provider(process.clone())?;

        Ok(DeterministicRuntime {
            runtime,
            console,
            filesystem,
            env,
            time,
            process,
        })
    }

    /// Registers a capability provider.
    pub fn register_provider<P>(&self, provider: P) -> Result<(), RuntimeError>
    where
        P: CapabilityProvider + 'static,
    {
        let name = provider.name().to_string();
        let mut providers = self
            .inner
            .providers
            .write()
            .expect("runtime providers poisoned");
        if providers.contains_key(&name) {
            return Err(RuntimeError::duplicate_provider(name));
        }
        providers.insert(name, Arc::new(provider));
        Ok(())
    }

    /// Enqueues a task for execution. Tasks are executed deterministically in
    /// FIFO order when [`Self::run`] is called.
    pub fn spawn(&self, spec: TaskSpec, plan: TaskPlan) {
        self.inner.scheduler.enqueue(TaskEntry { spec, plan });
    }

    /// Executes all pending tasks, yielding the runtime events emitted during
    /// execution.
    pub fn run(&self) -> Result<Vec<RuntimeEvent>, RuntimeError> {
        Ok(self.run_with_telemetry()?.into_events())
    }

    /// Executes all pending tasks and returns a structured trace capturing the
    /// runtime events alongside telemetry metadata such as event sequence
    /// numbers and coarse timestamps.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};
    /// let runtime = Runtime::with_default_shims().expect("runtime setup");
    /// let spec = TaskSpec::new("main").with_capabilities(["io"]);
    /// let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));
    /// runtime.spawn(spec, plan);
    /// let trace = runtime.run_with_telemetry().expect("runtime telemetry trace");
    /// assert_eq!(trace.events().len(), trace.telemetry().len());
    /// assert_eq!(trace.tasks()[0].task, "main");
    /// ```
    pub fn run_with_telemetry(&self) -> Result<RuntimeTrace, RuntimeError> {
        let mut events = Vec::new();
        let mut telemetry = Vec::new();
        let mut tasks = Vec::new();
        let mut sequence = 0usize;
        while let Some(entry) = self.inner.scheduler.pop() {
            let TaskExecutionResult {
                events: mut task_events,
                metrics,
            } = self.execute_task(&entry.spec, &entry.plan)?;
            for event in &task_events {
                telemetry.push(TelemetryEvent::new(sequence, event.clone()));
                sequence += 1;
            }
            events.append(&mut task_events);
            tasks.push(metrics);
        }
        Ok(RuntimeTrace {
            events,
            telemetry,
            tasks,
        })
    }

    /// Executes all pending tasks and returns a JSON representation of the
    /// runtime trace, suitable for downstream tooling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};
    /// let runtime = Runtime::with_default_shims().expect("runtime setup");
    /// let spec = TaskSpec::new("main").with_capabilities(["io"]);
    /// let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));
    /// runtime.spawn(spec, plan);
    /// let json = runtime
    ///     .run_with_trace_json()
    ///     .expect("runtime should serialize trace");
    /// assert!(json.contains("\"events\""));
    /// assert!(json.contains("\"task\":\"main\""));
    /// ```
    pub fn run_with_trace_json(&self) -> Result<String, RuntimeError> {
        self.run_with_telemetry()?.to_json_string()
    }

    /// Ensures all capabilities required by the provided task specification are
    /// registered with the runtime.
    pub fn ensure_capabilities(&self, spec: &TaskSpec) -> Result<(), RuntimeError> {
        for capability in spec.capabilities() {
            self.lookup_provider(capability)?;
        }
        Ok(())
    }

    fn execute_task(
        &self,
        spec: &TaskSpec,
        plan: &TaskPlan,
    ) -> Result<TaskExecutionResult, RuntimeError> {
        let mut events = Vec::new();
        let mut capability_counts: HashMap<String, usize> = HashMap::new();
        let mut operation_counts: HashMap<String, usize> = HashMap::new();
        let mut capability_durations: HashMap<String, u128> = HashMap::new();
        let mut operation_durations: HashMap<String, u128> = HashMap::new();
        let mut spawned_tasks = 0usize;
        let start_wall = SystemTime::now();
        let start = Instant::now();
        events.push(RuntimeEvent::TaskStarted {
            task: spec.name.clone(),
        });

        for op in plan.ops() {
            match op {
                TaskOp::Invoke {
                    capability,
                    operation,
                    payload,
                } => {
                    if !spec.has_capability(capability) {
                        return Err(RuntimeError::missing_capability(&spec.name, capability));
                    }
                    let provider = self.lookup_provider(capability)?;
                    *capability_counts.entry(capability.clone()).or_insert(0) += 1;
                    let op_key = format!("{}::{}", capability, operation);
                    *operation_counts.entry(op_key.clone()).or_insert(0) += 1;
                    events.push(RuntimeEvent::CapabilityInvoked {
                        task: spec.name.clone(),
                        capability: capability.clone(),
                        operation: operation.clone(),
                    });
                    let op_start = Instant::now();
                    let response = provider.handle(&CapabilityInvocation {
                        capability: capability.clone(),
                        operation: operation.clone(),
                        payload: payload.clone(),
                    })?;
                    let op_duration = op_start.elapsed().as_micros();
                    *capability_durations.entry(capability.clone()).or_insert(0) += op_duration;
                    *operation_durations.entry(op_key).or_insert(0) += op_duration;
                    for event in response.events {
                        events.push(RuntimeEvent::CapabilityEvent {
                            task: spec.name.clone(),
                            capability: capability.clone(),
                            event,
                        });
                    }
                }
                TaskOp::Spawn { spec: child, plan } => {
                    self.inner.scheduler.enqueue(TaskEntry {
                        spec: child.clone(),
                        plan: plan.clone(),
                    });
                    spawned_tasks += 1;
                    events.push(RuntimeEvent::TaskScheduled {
                        parent: spec.name.clone(),
                        child: child.name.clone(),
                    });
                }
            }
        }

        events.push(RuntimeEvent::TaskCompleted {
            task: spec.name.clone(),
        });
        let metrics = TaskTelemetry::new(
            spec.name.clone(),
            TaskTiming::new(start_wall, start.elapsed()),
            events.len(),
            TaskAggregates::new(
                capability_counts,
                operation_counts,
                capability_durations,
                operation_durations,
            ),
            spawned_tasks,
        );
        Ok(TaskExecutionResult { events, metrics })
    }

    fn lookup_provider(&self, name: &str) -> Result<Arc<dyn CapabilityProvider>, RuntimeError> {
        let providers = self
            .inner
            .providers
            .read()
            .expect("runtime providers poisoned");
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::unknown_capability(name))
    }
}

impl Scheduler {
    fn enqueue(&self, entry: TaskEntry) {
        self.queue
            .lock()
            .expect("scheduler queue poisoned")
            .push_back(entry);
    }

    fn pop(&self) -> Option<TaskEntry> {
        self.queue
            .lock()
            .expect("scheduler queue poisoned")
            .pop_front()
    }
}

struct TaskExecutionResult {
    events: Vec<RuntimeEvent>,
    metrics: TaskTelemetry,
}

/// Capability-oriented error surfaced by the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    DuplicateProvider { name: String },
    UnknownCapability { name: String },
    MissingCapability { task: String, capability: String },
    ProviderFailure { capability: String, message: String },
    SerializationFailure { message: String },
}

impl RuntimeError {
    fn duplicate_provider(name: String) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::DuplicateProvider { name },
        }
    }

    fn unknown_capability(name: &str) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::UnknownCapability {
                name: name.to_string(),
            },
        }
    }

    fn missing_capability(task: &str, capability: &str) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::MissingCapability {
                task: task.to_string(),
                capability: capability.to_string(),
            },
        }
    }

    pub fn provider_failure(capability: &str, message: impl Into<String>) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::ProviderFailure {
                capability: capability.to_string(),
                message: message.into(),
            },
        }
    }

    pub fn serialization(message: impl Into<String>) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::SerializationFailure {
                message: message.into(),
            },
        }
    }

    pub fn kind(&self) -> &RuntimeErrorKind {
        &self.kind
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RuntimeErrorKind::DuplicateProvider { name } => {
                write!(f, "capability provider '{name}' already registered")
            }
            RuntimeErrorKind::UnknownCapability { name } => {
                write!(f, "capability '{name}' is not registered with the runtime")
            }
            RuntimeErrorKind::MissingCapability { task, capability } => {
                write!(
                    f,
                    "task '{task}' attempted to use capability '{capability}' but it is not declared"
                )
            }
            RuntimeErrorKind::ProviderFailure {
                capability,
                message,
            } => {
                write!(
                    f,
                    "capability provider '{capability}' reported an error: {message}"
                )
            }
            RuntimeErrorKind::SerializationFailure { message } => {
                write!(f, "failed to serialize runtime trace: {message}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Runtime-level event emitted while executing tasks. The events are ordered in
/// the sequence they are observed by the scheduler.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEvent {
    TaskStarted {
        task: String,
    },
    CapabilityInvoked {
        task: String,
        capability: String,
        operation: String,
    },
    CapabilityEvent {
        task: String,
        capability: String,
        event: CapabilityEvent,
    },
    TaskScheduled {
        parent: String,
        child: String,
    },
    TaskCompleted {
        task: String,
    },
}

/// Structured telemetry metadata describing an observed runtime event.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryEvent {
    pub sequence: usize,
    pub timestamp_micros: Option<u128>,
    pub event: RuntimeEvent,
}

impl TelemetryEvent {
    fn new(sequence: usize, event: RuntimeEvent) -> Self {
        let timestamp_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_micros());
        TelemetryEvent {
            sequence,
            timestamp_micros,
            event,
        }
    }
}

/// Complete trace of a runtime execution, pairing raw events with their
/// telemetry representation.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTrace {
    events: Vec<RuntimeEvent>,
    telemetry: Vec<TelemetryEvent>,
    tasks: Vec<TaskTelemetry>,
}

impl RuntimeTrace {
    pub fn events(&self) -> &[RuntimeEvent] {
        &self.events
    }

    pub fn telemetry(&self) -> &[TelemetryEvent] {
        &self.telemetry
    }

    pub fn tasks(&self) -> &[TaskTelemetry] {
        &self.tasks
    }

    pub fn into_events(self) -> Vec<RuntimeEvent> {
        self.events
    }

    /// Produces an aggregated summary capturing total tasks, events, spawned
    /// children, and capability usage across the trace.
    pub fn summary(&self) -> RuntimeTraceSummary {
        let mut capability_totals: HashMap<String, usize> = HashMap::new();
        let mut operation_totals: HashMap<String, usize> = HashMap::new();
        let mut capability_duration_totals: HashMap<String, u128> = HashMap::new();
        let mut operation_duration_totals: HashMap<String, u128> = HashMap::new();
        let mut spawned = 0usize;
        for metrics in &self.tasks {
            spawned += metrics.spawned_tasks;
            for (capability, count) in &metrics.capability_counts {
                *capability_totals.entry(capability.clone()).or_insert(0) += *count;
            }
            for (operation, count) in &metrics.operation_counts {
                *operation_totals.entry(operation.clone()).or_insert(0) += *count;
            }
            for (capability, duration) in &metrics.capability_durations_micros {
                *capability_duration_totals
                    .entry(capability.clone())
                    .or_insert(0) += *duration;
            }
            for (operation, duration) in &metrics.operation_durations_micros {
                *operation_duration_totals
                    .entry(operation.clone())
                    .or_insert(0) += *duration;
            }
        }

        RuntimeTraceSummary {
            total_tasks: self.tasks.len(),
            total_events: self.events.len(),
            spawned_tasks: spawned,
            capability_counts: capability_totals,
            operation_counts: operation_totals,
            capability_durations_micros: capability_duration_totals,
            operation_durations_micros: operation_duration_totals,
        }
    }

    /// Serializes the runtime trace into a JSON object containing the events,
    /// telemetry timeline, per-task metrics, and a summary section.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mica::runtime::{Runtime, RuntimeValue, TaskPlan, TaskSpec};
    /// let runtime = Runtime::with_default_shims().expect("runtime setup");
    /// let spec = TaskSpec::new("main").with_capabilities(["io"]);
    /// let plan = TaskPlan::new().invoke("io", "write_line", Some(RuntimeValue::from("hello")));
    /// runtime.spawn(spec, plan);
    /// let trace = runtime
    ///     .run_with_telemetry()
    ///     .expect("runtime telemetry trace");
    /// let json = trace.to_json_string().expect("trace JSON");
    /// assert!(json.contains("\"telemetry\""));
    /// assert!(json.contains("\"tasks\""));
    /// ```
    pub fn to_json_string(&self) -> Result<String, RuntimeError> {
        let mut json = String::new();
        json.push('{');
        json.push_str("\"events\":[");
        for (index, event) in self.events.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&runtime_event_to_json(event));
        }
        json.push(']');
        json.push(',');
        json.push_str("\"telemetry\":[");
        for (index, entry) in self.telemetry.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&telemetry_event_to_json(entry));
        }
        json.push(']');
        json.push(',');
        json.push_str("\"tasks\":[");
        for (index, metrics) in self.tasks.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&task_telemetry_to_json(metrics));
        }
        json.push(']');
        json.push(',');
        json.push_str("\"summary\":");
        json.push_str(&runtime_trace_summary_to_json(&self.summary()));
        json.push('}');
        Ok(json)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTraceSummary {
    pub total_tasks: usize,
    pub total_events: usize,
    pub spawned_tasks: usize,
    pub capability_counts: HashMap<String, usize>,
    pub operation_counts: HashMap<String, usize>,
    pub capability_durations_micros: HashMap<String, u128>,
    pub operation_durations_micros: HashMap<String, u128>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskTelemetry {
    pub task: String,
    pub start_timestamp_micros: Option<u128>,
    pub duration_micros: u128,
    pub event_count: usize,
    pub capability_counts: HashMap<String, usize>,
    pub operation_counts: HashMap<String, usize>,
    pub capability_durations_micros: HashMap<String, u128>,
    pub operation_durations_micros: HashMap<String, u128>,
    pub spawned_tasks: usize,
}

impl TaskTelemetry {
    fn new(
        task: String,
        timing: TaskTiming,
        event_count: usize,
        aggregates: TaskAggregates,
        spawned_tasks: usize,
    ) -> Self {
        let start_timestamp_micros = timing
            .start
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_micros());
        TaskTelemetry {
            task,
            start_timestamp_micros,
            duration_micros: timing.duration.as_micros(),
            event_count,
            capability_counts: aggregates.capability_counts,
            operation_counts: aggregates.operation_counts,
            capability_durations_micros: aggregates.capability_durations_micros,
            operation_durations_micros: aggregates.operation_durations_micros,
            spawned_tasks,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TaskTiming {
    start: SystemTime,
    duration: Duration,
}

impl TaskTiming {
    fn new(start: SystemTime, duration: Duration) -> Self {
        TaskTiming { start, duration }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TaskAggregates {
    capability_counts: HashMap<String, usize>,
    operation_counts: HashMap<String, usize>,
    capability_durations_micros: HashMap<String, u128>,
    operation_durations_micros: HashMap<String, u128>,
}

impl TaskAggregates {
    fn new(
        capability_counts: HashMap<String, usize>,
        operation_counts: HashMap<String, usize>,
        capability_durations_micros: HashMap<String, u128>,
        operation_durations_micros: HashMap<String, u128>,
    ) -> Self {
        TaskAggregates {
            capability_counts,
            operation_counts,
            capability_durations_micros,
            operation_durations_micros,
        }
    }
}

/// Provider-specific event surfaced to the runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityEvent {
    Message(String),
    Data(RuntimeValue),
}

/// Primitive runtime values exchanged between capability providers and tasks.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl RuntimeValue {
    pub fn unit() -> Self {
        RuntimeValue::Unit
    }

    pub fn string(value: impl Into<String>) -> Self {
        RuntimeValue::String(value.into())
    }
}

impl From<i64> for RuntimeValue {
    fn from(value: i64) -> Self {
        RuntimeValue::Int(value)
    }
}

impl From<bool> for RuntimeValue {
    fn from(value: bool) -> Self {
        RuntimeValue::Bool(value)
    }
}

impl From<f64> for RuntimeValue {
    fn from(value: f64) -> Self {
        RuntimeValue::Float(value)
    }
}

impl From<String> for RuntimeValue {
    fn from(value: String) -> Self {
        RuntimeValue::String(value)
    }
}

impl<'a> From<&'a str> for RuntimeValue {
    fn from(value: &'a str) -> Self {
        RuntimeValue::String(value.to_string())
    }
}

fn telemetry_event_to_json(entry: &TelemetryEvent) -> String {
    let mut json = String::new();
    json.push('{');
    json.push_str("\"sequence\":");
    json.push_str(&entry.sequence.to_string());
    json.push(',');
    json.push_str("\"timestamp_micros\":");
    if let Some(value) = entry.timestamp_micros {
        json.push_str(&value.to_string());
    } else {
        json.push_str("null");
    }
    json.push(',');
    json.push_str("\"event\":");
    json.push_str(&runtime_event_to_json(&entry.event));
    json.push('}');
    json
}

fn task_telemetry_to_json(metrics: &TaskTelemetry) -> String {
    let mut json = String::new();
    json.push('{');
    json.push_str("\"task\":\"");
    json.push_str(&escape_json_string(&metrics.task));
    json.push_str("\",");
    json.push_str("\"start_timestamp_micros\":");
    if let Some(value) = metrics.start_timestamp_micros {
        json.push_str(&value.to_string());
    } else {
        json.push_str("null");
    }
    json.push(',');
    json.push_str("\"duration_micros\":");
    json.push_str(&metrics.duration_micros.to_string());
    json.push(',');
    json.push_str("\"event_count\":");
    json.push_str(&metrics.event_count.to_string());
    json.push(',');
    json.push_str("\"capability_counts\":");
    json.push_str(&capability_counts_to_json(&metrics.capability_counts));
    json.push(',');
    json.push_str("\"operation_counts\":");
    json.push_str(&capability_counts_to_json(&metrics.operation_counts));
    json.push(',');
    json.push_str("\"capability_durations_micros\":");
    json.push_str(&duration_map_to_json(&metrics.capability_durations_micros));
    json.push(',');
    json.push_str("\"operation_durations_micros\":");
    json.push_str(&duration_map_to_json(&metrics.operation_durations_micros));
    json.push(',');
    json.push_str("\"spawned_tasks\":");
    json.push_str(&metrics.spawned_tasks.to_string());
    json.push('}');
    json
}

fn runtime_trace_summary_to_json(summary: &RuntimeTraceSummary) -> String {
    let mut json = String::new();
    json.push('{');
    json.push_str("\"total_tasks\":");
    json.push_str(&summary.total_tasks.to_string());
    json.push(',');
    json.push_str("\"total_events\":");
    json.push_str(&summary.total_events.to_string());
    json.push(',');
    json.push_str("\"spawned_tasks\":");
    json.push_str(&summary.spawned_tasks.to_string());
    json.push(',');
    json.push_str("\"capability_counts\":");
    json.push_str(&capability_counts_to_json(&summary.capability_counts));
    json.push(',');
    json.push_str("\"operation_counts\":");
    json.push_str(&capability_counts_to_json(&summary.operation_counts));
    json.push(',');
    json.push_str("\"capability_durations_micros\":");
    json.push_str(&duration_map_to_json(&summary.capability_durations_micros));
    json.push(',');
    json.push_str("\"operation_durations_micros\":");
    json.push_str(&duration_map_to_json(&summary.operation_durations_micros));
    json.push('}');
    json
}

fn capability_counts_to_json(map: &HashMap<String, usize>) -> String {
    if map.is_empty() {
        return "{}".to_string();
    }
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut json = String::from("{");
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&escape_json_string(key));
        json.push('"');
        json.push(':');
        json.push_str(&value.to_string());
    }
    json.push('}');
    json
}

fn duration_map_to_json(map: &HashMap<String, u128>) -> String {
    if map.is_empty() {
        return "{}".to_string();
    }
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut json = String::from("{");
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&escape_json_string(key));
        json.push('"');
        json.push(':');
        json.push_str(&value.to_string());
    }
    json.push('}');
    json
}

fn runtime_event_to_json(event: &RuntimeEvent) -> String {
    match event {
        RuntimeEvent::TaskStarted { task } => format!(
            "{{\"type\":\"task_started\",\"task\":\"{}\"}}",
            escape_json_string(task)
        ),
        RuntimeEvent::CapabilityInvoked {
            task,
            capability,
            operation,
        } => format!(
            "{{\"type\":\"capability_invoked\",\"task\":\"{}\",\"capability\":\"{}\",\"operation\":\"{}\"}}",
            escape_json_string(task),
            escape_json_string(capability),
            escape_json_string(operation)
        ),
        RuntimeEvent::CapabilityEvent {
            task,
            capability,
            event,
        } => format!(
            "{{\"type\":\"capability_event\",\"task\":\"{}\",\"capability\":\"{}\",\"event\":{}}}",
            escape_json_string(task),
            escape_json_string(capability),
            capability_event_to_json(event)
        ),
        RuntimeEvent::TaskScheduled { parent, child } => format!(
            "{{\"type\":\"task_scheduled\",\"parent\":\"{}\",\"child\":\"{}\"}}",
            escape_json_string(parent),
            escape_json_string(child)
        ),
        RuntimeEvent::TaskCompleted { task } => format!(
            "{{\"type\":\"task_completed\",\"task\":\"{}\"}}",
            escape_json_string(task)
        ),
    }
}

fn capability_event_to_json(event: &CapabilityEvent) -> String {
    match event {
        CapabilityEvent::Message(value) => format!(
            "{{\"type\":\"message\",\"value\":\"{}\"}}",
            escape_json_string(value)
        ),
        CapabilityEvent::Data(value) => format!(
            "{{\"type\":\"data\",\"value\":{}}}",
            runtime_value_to_json(value)
        ),
    }
}

fn runtime_value_to_json(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::Unit => "null".to_string(),
        RuntimeValue::Int(value) => value.to_string(),
        RuntimeValue::Float(value) => {
            if value.is_finite() {
                value.to_string()
            } else if value.is_nan() {
                "\"NaN\"".to_string()
            } else if value.is_sign_positive() {
                "\"Infinity\"".to_string()
            } else {
                "\"-Infinity\"".to_string()
            }
        }
        RuntimeValue::Bool(value) => value.to_string(),
        RuntimeValue::String(value) => {
            format!("\"{}\"", escape_json_string(value))
        }
    }
}

fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                use std::fmt::Write;
                write!(&mut escaped, "\\u{:04X}", ch as u32).expect("json escape write");
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Capability provider interface implemented by concrete runtime shims.
pub trait CapabilityProvider: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError>;
}

/// Request delivered to capability providers.
#[derive(Debug, Clone)]
pub struct CapabilityInvocation {
    pub capability: String,
    pub operation: String,
    pub payload: Option<RuntimeValue>,
}

/// Provider response containing the return value and emitted events.
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub value: RuntimeValue,
    pub events: Vec<CapabilityEvent>,
}

impl ProviderResponse {
    pub fn new(value: RuntimeValue) -> Self {
        ProviderResponse {
            value,
            events: Vec::new(),
        }
    }

    pub fn with_event(mut self, event: CapabilityEvent) -> Self {
        self.events.push(event);
        self
    }
}

/// Declarative task description consumed by the runtime scheduler.
#[derive(Debug, Clone, Default)]
pub struct TaskPlan {
    ops: Vec<TaskOp>,
}

impl TaskPlan {
    pub fn new() -> Self {
        TaskPlan { ops: Vec::new() }
    }

    pub fn invoke(
        mut self,
        capability: impl Into<String>,
        operation: impl Into<String>,
        payload: Option<RuntimeValue>,
    ) -> Self {
        self.ops.push(TaskOp::Invoke {
            capability: capability.into(),
            operation: operation.into(),
            payload,
        });
        self
    }

    pub fn spawn(mut self, spec: TaskSpec, plan: TaskPlan) -> Self {
        self.ops.push(TaskOp::Spawn { spec, plan });
        self
    }

    fn ops(&self) -> &[TaskOp] {
        &self.ops
    }
}

#[derive(Debug, Clone)]
enum TaskOp {
    Invoke {
        capability: String,
        operation: String,
        payload: Option<RuntimeValue>,
    },
    Spawn {
        spec: TaskSpec,
        plan: TaskPlan,
    },
}

/// Task metadata describing the capability requirements recorded in the
/// compiler's effect rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSpec {
    name: String,
    capabilities: Vec<String>,
}

impl TaskSpec {
    pub fn new(name: impl Into<String>) -> Self {
        TaskSpec {
            name: name.into(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capabilities(
        mut self,
        capabilities: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for capability in capabilities {
            self.require(capability);
        }
        self
    }

    pub fn require(&mut self, capability: impl Into<String>) {
        let capability = capability.into();
        if !self.capabilities.iter().any(|c| c == &capability) {
            self.capabilities.push(capability);
        }
    }

    fn has_capability(&self, name: &str) -> bool {
        self.capabilities.iter().any(|cap| cap == name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

#[derive(Debug, Clone)]
pub struct CompletedProcess {
    pub command: String,
    pub exit_code: i32,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScriptedProcess {
    command: String,
    stdout: Vec<String>,
    stderr: Vec<String>,
    exit_code: i32,
}

impl ScriptedProcess {
    pub fn new(command: impl Into<String>) -> Self {
        ScriptedProcess {
            command: command.into(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    pub fn with_stdout_line(mut self, line: impl Into<String>) -> Self {
        self.stdout.push(line.into());
        self
    }

    pub fn with_stderr_line(mut self, line: impl Into<String>) -> Self {
        self.stderr.push(line.into());
        self
    }
}

#[derive(Debug, Default)]
struct DeterministicProcessInner {
    scripted: Mutex<VecDeque<ScriptedProcess>>,
    completed: Mutex<Vec<CompletedProcess>>,
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicProcessProvider {
    inner: Arc<DeterministicProcessInner>,
}

impl DeterministicProcessProvider {
    pub fn script(&self, process: ScriptedProcess) {
        self.inner
            .scripted
            .lock()
            .expect("deterministic process queue poisoned")
            .push_back(process);
    }

    pub fn completed(&self) -> Vec<CompletedProcess> {
        self.inner
            .completed
            .lock()
            .expect("deterministic process results poisoned")
            .clone()
    }

    fn dequeue(&self) -> Option<ScriptedProcess> {
        self.inner
            .scripted
            .lock()
            .expect("deterministic process queue poisoned")
            .pop_front()
    }

    fn push_completed(&self, process: &ScriptedProcess) {
        self.inner
            .completed
            .lock()
            .expect("deterministic process results poisoned")
            .push(CompletedProcess {
                command: process.command.clone(),
                exit_code: process.exit_code,
                stdout: process.stdout.clone(),
                stderr: process.stderr.clone(),
            });
    }
}

impl CapabilityProvider for DeterministicProcessProvider {
    fn name(&self) -> &str {
        "process"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "spawn" => {
                let requested = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "spawn expects a string payload",
                        )
                    })?;

                let scripted = self.dequeue().ok_or_else(|| {
                    RuntimeError::provider_failure(
                        self.name(),
                        "spawn invoked without scripted process",
                    )
                })?;

                if scripted.command != requested {
                    return Err(RuntimeError::provider_failure(
                        self.name(),
                        format!(
                            "expected scripted command '{}' but received '{}'",
                            scripted.command, requested
                        ),
                    ));
                }

                self.push_completed(&scripted);

                let mut response =
                    ProviderResponse::new(RuntimeValue::from(i64::from(scripted.exit_code)));
                response = response.with_event(CapabilityEvent::Message(format!(
                    "spawned {}",
                    scripted.command
                )));
                for line in &scripted.stdout {
                    response =
                        response.with_event(CapabilityEvent::Message(format!("stdout: {}", line)));
                }
                for line in &scripted.stderr {
                    response =
                        response.with_event(CapabilityEvent::Message(format!("stderr: {}", line)));
                }
                response = response.with_event(CapabilityEvent::Data(RuntimeValue::from(
                    i64::from(scripted.exit_code),
                )));
                Ok(response)
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicConsoleProvider {
    inner: Arc<DeterministicConsoleInner>,
}

#[derive(Debug, Default)]
struct DeterministicConsoleInner {
    writes: Mutex<Vec<String>>,
    inputs: Mutex<VecDeque<String>>,
}

impl DeterministicConsoleProvider {
    pub fn with_inputs<I, S>(inputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let provider = DeterministicConsoleProvider::default();
        {
            let mut guard = provider
                .inner
                .inputs
                .lock()
                .expect("console inputs poisoned");
            guard.extend(inputs.into_iter().map(Into::into));
        }
        provider
    }

    pub fn queue_input(&self, value: impl Into<String>) {
        self.inner
            .inputs
            .lock()
            .expect("console inputs poisoned")
            .push_back(value.into());
    }

    pub fn writes(&self) -> Vec<String> {
        self.inner
            .writes
            .lock()
            .expect("console writes poisoned")
            .clone()
    }

    pub fn clear_writes(&self) {
        self.inner
            .writes
            .lock()
            .expect("console writes poisoned")
            .clear();
    }
}

impl CapabilityProvider for DeterministicConsoleProvider {
    fn name(&self) -> &str {
        "io"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "write_line" => {
                let message = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_line expects a string payload",
                        )
                    })?;
                self.inner
                    .writes
                    .lock()
                    .expect("console writes poisoned")
                    .push(message.clone());
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(message)))
            }
            "read_line" => {
                let line = self
                    .inner
                    .inputs
                    .lock()
                    .expect("console inputs poisoned")
                    .pop_front()
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "read_line requires scripted input",
                        )
                    })?;
                Ok(ProviderResponse::new(RuntimeValue::from(line.clone()))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(line))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryFilesystemProvider {
    files: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryFilesystemProvider {
    pub fn write_file(&self, path: impl Into<String>, contents: impl Into<String>) {
        self.files
            .write()
            .expect("filesystem store poisoned")
            .insert(path.into(), contents.into());
    }

    pub fn read_file(&self, path: &str) -> Option<String> {
        self.files
            .read()
            .expect("filesystem store poisoned")
            .get(path)
            .cloned()
    }

    pub fn snapshot(&self) -> HashMap<String, String> {
        self.files
            .read()
            .expect("filesystem store poisoned")
            .clone()
    }
}

impl CapabilityProvider for InMemoryFilesystemProvider {
    fn name(&self) -> &str {
        "fs"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "read_to_string" => {
                let path = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(path) => Some(path.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "read_to_string expects a string payload",
                        )
                    })?;
                let contents = self
                    .files
                    .read()
                    .expect("filesystem store poisoned")
                    .get(&path)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            format!("no in-memory file registered for '{path}'"),
                        )
                    })?;
                Ok(ProviderResponse::new(RuntimeValue::from(contents.clone()))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(contents))))
            }
            "write_string" => {
                let path = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(path) => Some(path.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string expects a string payload",
                        )
                    })?;
                let mut segments = path.splitn(2, '=');
                let file_path = segments
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string payload must be 'path=contents'",
                        )
                    })?;
                let contents = segments
                    .next()
                    .map(|value| value.to_string())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string payload must include file contents",
                        )
                    })?;
                self.write_file(file_path.clone(), contents.clone());
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("wrote {file_path}"))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicEnvProvider {
    values: Arc<RwLock<HashMap<String, String>>>,
}

impl DeterministicEnvProvider {
    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        self.values
            .write()
            .expect("env store poisoned")
            .insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.values
            .read()
            .expect("env store poisoned")
            .get(key)
            .cloned()
    }

    pub fn unset(&self, key: &str) {
        self.values.write().expect("env store poisoned").remove(key);
    }

    pub fn snapshot(&self) -> HashMap<String, String> {
        self.values.read().expect("env store poisoned").clone()
    }
}

impl CapabilityProvider for DeterministicEnvProvider {
    fn name(&self) -> &str {
        "env"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "get" => {
                let key = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(self.name(), "get expects a string payload")
                    })?;
                let value = self
                    .values
                    .read()
                    .expect("env store poisoned")
                    .get(&key)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            format!("environment variable '{key}' is not scripted"),
                        )
                    })?;
                Ok(ProviderResponse::new(RuntimeValue::from(value.clone()))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(value))))
            }
            "set" => {
                let assignment = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(self.name(), "set expects a string payload")
                    })?;
                let mut parts = assignment.splitn(2, '=');
                let key = parts
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "set payload must be 'KEY=VALUE'",
                        )
                    })?;
                let value = parts.next().ok_or_else(|| {
                    RuntimeError::provider_failure(self.name(), "set payload must include a value")
                })?;
                self.set(key.clone(), value);
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("set {key}"))))
            }
            "unset" => {
                let key = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "unset expects a string payload",
                        )
                    })?;
                self.unset(&key);
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("unset {key}"))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeterministicTimeProvider {
    state: Arc<DeterministicTimeState>,
}

#[derive(Debug)]
struct DeterministicTimeState {
    inner: Mutex<DeterministicTimeInner>,
}

#[derive(Debug)]
struct DeterministicTimeInner {
    next: i64,
    step: i64,
    scripted: VecDeque<i64>,
    last_emitted: Option<i64>,
}

impl Default for DeterministicTimeProvider {
    fn default() -> Self {
        Self::monotonic(0, 1)
    }
}

impl DeterministicTimeProvider {
    pub fn monotonic(start: i64, step: i64) -> Self {
        DeterministicTimeProvider {
            state: Arc::new(DeterministicTimeState {
                inner: Mutex::new(DeterministicTimeInner {
                    next: start,
                    step,
                    scripted: VecDeque::new(),
                    last_emitted: None,
                }),
            }),
        }
    }

    pub fn scripted<I>(values: I) -> Self
    where
        I: IntoIterator<Item = i64>,
    {
        let provider = DeterministicTimeProvider::monotonic(0, 1);
        {
            let mut guard = provider.state.inner.lock().expect("time state poisoned");
            guard.scripted = values.into_iter().collect();
        }
        provider
    }

    pub fn push_time(&self, value: i64) {
        self.state
            .inner
            .lock()
            .expect("time state poisoned")
            .scripted
            .push_back(value);
    }

    pub fn set_step(&self, step: i64) {
        self.state.inner.lock().expect("time state poisoned").step = step;
    }

    pub fn last_emitted(&self) -> Option<i64> {
        self.state
            .inner
            .lock()
            .expect("time state poisoned")
            .last_emitted
    }

    fn next_timestamp(&self) -> i64 {
        let mut guard = self.state.inner.lock().expect("time state poisoned");
        let value = if let Some(scripted) = guard.scripted.pop_front() {
            guard.next = scripted.saturating_add(guard.step);
            scripted
        } else {
            let current = guard.next;
            guard.next = guard.next.saturating_add(guard.step);
            current
        };
        guard.last_emitted = Some(value);
        value
    }
}

impl CapabilityProvider for DeterministicTimeProvider {
    fn name(&self) -> &str {
        "time"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "now_millis" => {
                let value = self.next_timestamp();
                Ok(ProviderResponse::new(RuntimeValue::from(value))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(value))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Default)]
struct ConsoleProvider;

impl CapabilityProvider for ConsoleProvider {
    fn name(&self) -> &str {
        "io"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "write_line" => {
                let message = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_line expects a string payload",
                        )
                    })?;
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(message)))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Default)]
struct FilesystemProvider;

impl CapabilityProvider for FilesystemProvider {
    fn name(&self) -> &str {
        "fs"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "read_to_string" => {
                let path = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(path) => Some(path.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "read_to_string expects a string payload",
                        )
                    })?;
                let contents = fs::read_to_string(&path).map_err(|err| {
                    RuntimeError::provider_failure(
                        self.name(),
                        format!("failed to read '{path}': {err}"),
                    )
                })?;
                Ok(ProviderResponse::new(RuntimeValue::from(contents.clone()))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(contents))))
            }
            "write_string" => {
                let path = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(path) => Some(path.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string expects a string payload",
                        )
                    })?;
                let mut segments = path.splitn(2, '=');
                let file_path = segments
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string payload must be 'path=contents'",
                        )
                    })?;
                let contents = segments
                    .next()
                    .map(|value| value.to_string())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "write_string payload must include file contents",
                        )
                    })?;
                fs::write(&file_path, &contents).map_err(|err| {
                    RuntimeError::provider_failure(
                        self.name(),
                        format!("failed to write '{file_path}': {err}"),
                    )
                })?;
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("wrote {file_path}"))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Default)]
struct NetworkProvider;

impl NetworkProvider {
    fn load_fixture(&self, key: &str) -> Option<NetworkFixture> {
        network_fixtures()
            .read()
            .expect("network fixtures poisoned")
            .get(key)
            .cloned()
    }
}

impl CapabilityProvider for NetworkProvider {
    fn name(&self) -> &str {
        "net"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "fetch" => {
                let key = invocation
                    .payload
                    .as_ref()
                    .and_then(|value| match value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "fetch expects a string payload identifying a fixture",
                        )
                    })?;

                let fixture = self.load_fixture(&key).ok_or_else(|| {
                    RuntimeError::provider_failure(
                        self.name(),
                        format!("no fixture registered for '{key}'"),
                    )
                })?;

                let mut response = ProviderResponse::new(RuntimeValue::from(fixture.body.clone()))
                    .with_event(CapabilityEvent::Message(format!(
                        "fixture {key} responded with status {}",
                        fixture.status
                    )))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(i64::from(
                        fixture.status,
                    ))));

                for (name, value) in fixture.sorted_headers() {
                    response = response
                        .with_event(CapabilityEvent::Message(format!("header {name}: {value}")));
                }

                Ok(response)
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFixture {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

impl NetworkFixture {
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        NetworkFixture {
            status,
            body: body.into(),
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    fn sorted_headers(&self) -> Vec<(String, String)> {
        let mut headers = self.headers.clone();
        headers.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        headers
    }
}

pub fn register_network_fixture(key: impl Into<String>, fixture: NetworkFixture) {
    network_fixtures()
        .write()
        .expect("network fixtures poisoned")
        .insert(key.into(), fixture);
}

pub fn reset_network_fixtures() {
    network_fixtures()
        .write()
        .expect("network fixtures poisoned")
        .clear();
}

fn network_fixtures() -> &'static RwLock<HashMap<String, NetworkFixture>> {
    static FIXTURES: OnceLock<RwLock<HashMap<String, NetworkFixture>>> = OnceLock::new();
    FIXTURES.get_or_init(|| RwLock::new(HashMap::new()))
}

#[derive(Debug, Default)]
struct ProcessProvider;

impl ProcessProvider {
    fn parse_command(
        &self,
        invocation: &CapabilityInvocation,
    ) -> Result<Vec<String>, RuntimeError> {
        let payload = invocation
            .payload
            .as_ref()
            .and_then(|value| match value {
                RuntimeValue::String(value) => Some(value.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                RuntimeError::provider_failure(self.name(), "spawn expects a string payload")
            })?;

        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = payload.chars().peekable();
        let mut in_quotes = false;
        let mut quote = '\0';
        while let Some(ch) = chars.next() {
            match ch {
                '\'' | '"' => {
                    if in_quotes {
                        if ch == quote {
                            in_quotes = false;
                        } else {
                            current.push(ch);
                        }
                    } else {
                        in_quotes = true;
                        quote = ch;
                    }
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                }
                ch if ch.is_whitespace() && !in_quotes => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if in_quotes {
            return Err(RuntimeError::provider_failure(
                self.name(),
                "unterminated quote in process payload",
            ));
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            return Err(RuntimeError::provider_failure(
                self.name(),
                "spawn requires a command to execute",
            ));
        }

        Ok(parts)
    }

    fn spawn(&self, args: &[String]) -> Result<std::process::Output, RuntimeError> {
        let mut command = Command::new(&args[0]);
        if args.len() > 1 {
            command.args(&args[1..]);
        }
        command.output().map_err(|err| {
            RuntimeError::provider_failure(
                self.name(),
                format!("failed to spawn '{}': {err}", args[0]),
            )
        })
    }

    fn append_stream_events(
        &self,
        response: ProviderResponse,
        stdout: &[u8],
        stderr: &[u8],
    ) -> ProviderResponse {
        let mut response = response;
        let stdout_text = String::from_utf8_lossy(stdout);
        for line in stdout_text.lines() {
            if !line.is_empty() {
                response =
                    response.with_event(CapabilityEvent::Message(format!("stdout: {line}",)));
            }
        }
        let stderr_text = String::from_utf8_lossy(stderr);
        for line in stderr_text.lines() {
            if !line.is_empty() {
                response =
                    response.with_event(CapabilityEvent::Message(format!("stderr: {line}",)));
            }
        }
        response
    }
}

impl CapabilityProvider for ProcessProvider {
    fn name(&self) -> &str {
        "process"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "spawn" => {
                let args = self.parse_command(invocation)?;
                let output = self.spawn(&args)?;
                let exit_code = output.status.code().ok_or_else(|| {
                    RuntimeError::provider_failure(
                        self.name(),
                        "process exited without status code",
                    )
                })?;
                let mut response =
                    ProviderResponse::new(RuntimeValue::from(i64::from(exit_code))).with_event(
                        CapabilityEvent::Message(format!("spawned {}", args.join(" "))),
                    );
                response = self.append_stream_events(response, &output.stdout, &output.stderr);
                Ok(
                    response.with_event(CapabilityEvent::Data(RuntimeValue::from(i64::from(
                        exit_code,
                    )))),
                )
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Default)]
struct EnvProvider;

impl EnvProvider {
    fn string_payload(
        &self,
        invocation: &CapabilityInvocation,
        operation: &str,
    ) -> Result<String, RuntimeError> {
        invocation
            .payload
            .as_ref()
            .and_then(|value| match value {
                RuntimeValue::String(value) => Some(value.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                RuntimeError::provider_failure(
                    self.name(),
                    format!("{operation} expects a string payload"),
                )
            })
    }
}

impl CapabilityProvider for EnvProvider {
    fn name(&self) -> &str {
        "env"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "get" => {
                let key = self.string_payload(invocation, "get")?;
                match env::var(&key) {
                    Ok(value) => Ok(ProviderResponse::new(RuntimeValue::from(value.clone()))
                        .with_event(CapabilityEvent::Data(RuntimeValue::from(value)))),
                    Err(_) => Err(RuntimeError::provider_failure(
                        self.name(),
                        format!("environment variable '{key}' is not set"),
                    )),
                }
            }
            "set" => {
                let assignment = self.string_payload(invocation, "set")?;
                let mut parts = assignment.splitn(2, '=');
                let key = parts
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        RuntimeError::provider_failure(
                            self.name(),
                            "set payload must be 'KEY=VALUE'",
                        )
                    })?;
                let value = parts.next().ok_or_else(|| {
                    RuntimeError::provider_failure(self.name(), "set payload must include a value")
                })?;
                let value = value.to_string();
                // SAFETY: The runtime only writes UTF-8 key/value pairs provided by the
                // compiled program, mirroring the behaviour of the previous implementation.
                unsafe {
                    env::set_var(&key, &value);
                }
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("set {key}"))))
            }
            "unset" => {
                let key = self.string_payload(invocation, "unset")?;
                // SAFETY: Removing the variable simply mirrors the task request.
                unsafe {
                    env::remove_var(&key);
                }
                Ok(ProviderResponse::new(RuntimeValue::unit())
                    .with_event(CapabilityEvent::Message(format!("unset {key}"))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

#[derive(Debug, Default)]
struct TimeProvider;

impl CapabilityProvider for TimeProvider {
    fn name(&self) -> &str {
        "time"
    }

    fn handle(&self, invocation: &CapabilityInvocation) -> Result<ProviderResponse, RuntimeError> {
        match invocation.operation.as_str() {
            "now_millis" => {
                let millis: i64 = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    .try_into()
                    .unwrap_or(i64::MAX);
                Ok(ProviderResponse::new(RuntimeValue::from(millis))
                    .with_event(CapabilityEvent::Data(RuntimeValue::from(millis))))
            }
            other => Err(RuntimeError::provider_failure(
                self.name(),
                format!("unsupported operation '{other}'"),
            )),
        }
    }
}

impl From<RuntimeError> for BackendError {
    fn from(err: RuntimeError) -> Self {
        BackendError::Internal(err.to_string())
    }
}
