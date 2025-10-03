use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::backend::BackendError;

/// Runtime orchestrator that maps declared capability requirements to concrete
/// providers and executes task plans in a deterministic FIFO order.
#[derive(Debug, Clone, Default)]
pub struct Runtime {
    inner: Arc<RuntimeInner>,
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

    /// Registers the stock providers used in the language tour (console IO and
    /// wall-clock time) so capability-driven examples can execute out of the box.
    pub fn with_default_shims() -> Result<Self, RuntimeError> {
        let runtime = Runtime::new();
        runtime.register_provider(ConsoleProvider)?;
        runtime.register_provider(TimeProvider)?;
        runtime.register_provider(FilesystemProvider)?;
        Ok(runtime)
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
    pub fn run_with_telemetry(&self) -> Result<RuntimeTrace, RuntimeError> {
        let mut events = Vec::new();
        let mut telemetry = Vec::new();
        let mut sequence = 0usize;
        while let Some(entry) = self.inner.scheduler.pop() {
            let mut task_events = self.execute_task(&entry.spec, &entry.plan)?;
            for event in &task_events {
                telemetry.push(TelemetryEvent::new(sequence, event.clone()));
                sequence += 1;
            }
            events.append(&mut task_events);
        }
        Ok(RuntimeTrace { events, telemetry })
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
    ) -> Result<Vec<RuntimeEvent>, RuntimeError> {
        let mut events = Vec::new();
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
                    events.push(RuntimeEvent::CapabilityInvoked {
                        task: spec.name.clone(),
                        capability: capability.clone(),
                        operation: operation.clone(),
                    });
                    let response = provider.handle(&CapabilityInvocation {
                        capability: capability.clone(),
                        operation: operation.clone(),
                        payload: payload.clone(),
                    })?;
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
        Ok(events)
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
}

impl RuntimeTrace {
    pub fn events(&self) -> &[RuntimeEvent] {
        &self.events
    }

    pub fn telemetry(&self) -> &[TelemetryEvent] {
        &self.telemetry
    }

    pub fn into_events(self) -> Vec<RuntimeEvent> {
        self.events
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
