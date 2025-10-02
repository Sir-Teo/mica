use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::ir;

pub mod llvm;
pub mod native;
pub mod text;

#[derive(Debug, Clone, Default)]
pub struct BackendOptions {
    pub optimize: bool,
    pub debug_info: bool,
    pub target_triple: Option<String>,
}

pub trait Backend {
    type Output;

    fn compile(&self, module: &ir::Module, options: &BackendOptions)
    -> BackendResult<Self::Output>;
}

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone)]
pub enum BackendError {
    Unsupported(String),
    Internal(String),
}

impl BackendError {
    pub fn unsupported<T: Into<String>>(message: T) -> Self {
        BackendError::Unsupported(message.into())
    }

    pub fn internal<T: Into<String>>(message: T) -> Self {
        BackendError::Internal(message.into())
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::Unsupported(msg) => write!(f, "unsupported backend feature: {}", msg),
            BackendError::Internal(msg) => write!(f, "backend error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

pub fn run<B: Backend>(
    backend: &B,
    module: &ir::Module,
    options: &BackendOptions,
) -> BackendResult<B::Output> {
    backend.compile(module, options)
}

#[derive(Debug, Clone)]
pub struct ParallelCompileReport<T> {
    pub outputs: Vec<T>,
    pub metrics: ParallelCompileMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct ParallelCompileMetrics {
    pub total_duration: Duration,
    pub modules: Vec<ModuleCompileMetrics>,
}

#[derive(Debug, Clone)]
pub struct ModuleCompileMetrics {
    pub module: String,
    pub duration: Duration,
}

pub fn run_parallel<B>(
    backend: &B,
    modules: &[ir::Module],
    options: &BackendOptions,
) -> BackendResult<ParallelCompileReport<B::Output>>
where
    B: Backend + Sync,
    B::Output: Send + 'static,
{
    if modules.is_empty() {
        return Ok(ParallelCompileReport {
            outputs: Vec::new(),
            metrics: ParallelCompileMetrics::default(),
        });
    }

    let mut output_slots = Vec::with_capacity(modules.len());
    output_slots.resize_with(modules.len(), || None);
    let outputs: Arc<Mutex<Vec<Option<B::Output>>>> = Arc::new(Mutex::new(output_slots));

    let mut duration_slots = Vec::with_capacity(modules.len());
    duration_slots.resize_with(modules.len(), || None);
    let durations: Arc<Mutex<Vec<Option<Duration>>>> = Arc::new(Mutex::new(duration_slots));
    let error: Arc<Mutex<Option<BackendError>>> = Arc::new(Mutex::new(None));
    let options = options.clone();
    let start = Instant::now();

    let worker_count = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(modules.len())
        .max(1);
    let next_index = Arc::new(AtomicUsize::new(0));

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let outputs = Arc::clone(&outputs);
            let durations = Arc::clone(&durations);
            let error = Arc::clone(&error);
            let options = options.clone();
            let next_index = Arc::clone(&next_index);
            scope.spawn(move || {
                loop {
                    if error.lock().unwrap().is_some() {
                        return;
                    }

                    let index = next_index.fetch_add(1, Ordering::SeqCst);
                    if index >= modules.len() {
                        return;
                    }

                    let module = &modules[index];
                    let module_start = Instant::now();
                    match backend.compile(module, &options) {
                        Ok(result) => {
                            durations.lock().unwrap()[index] = Some(module_start.elapsed());
                            outputs.lock().unwrap()[index] = Some(result);
                        }
                        Err(err) => {
                            let mut guard = error.lock().unwrap();
                            if guard.is_none() {
                                *guard = Some(err);
                            }
                        }
                    }
                }
            });
        }
    });

    if let Some(err) = error.lock().unwrap().take() {
        return Err(err);
    }

    let outputs = outputs
        .lock()
        .unwrap()
        .iter_mut()
        .map(|entry| entry.take().expect("missing backend output"))
        .collect::<Vec<_>>();

    let duration_guard = durations.lock().unwrap();
    let module_metrics = modules
        .iter()
        .enumerate()
        .map(|(index, module)| ModuleCompileMetrics {
            module: if module.name.is_empty() {
                "<root>".to_string()
            } else {
                module.name.join("::")
            },
            duration: duration_guard[index].unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    let metrics = ParallelCompileMetrics {
        total_duration: start.elapsed(),
        modules: module_metrics,
    };

    Ok(ParallelCompileReport { outputs, metrics })
}
