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
    pub worker_count: usize,
    pub scheduled_modules: usize,
    pub worker_metrics: Vec<WorkerMetrics>,
    pub schedule: Vec<ScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct ModuleCompileMetrics {
    pub module: String,
    pub duration: Duration,
    pub worker_index: usize,
    pub start_offset: Duration,
}

#[derive(Debug, Clone)]
pub struct WorkerMetrics {
    pub worker_index: usize,
    pub processed_modules: usize,
    pub busy_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub module: String,
    pub worker_index: usize,
    pub start_offset: Duration,
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
    #[derive(Default, Clone)]
    struct ModuleRecord {
        worker_index: usize,
        duration: Duration,
        start_offset: Duration,
    }

    #[derive(Default)]
    struct WorkerAccumulator {
        processed_modules: usize,
        busy_duration: Duration,
    }

    if modules.is_empty() {
        return Ok(ParallelCompileReport {
            outputs: Vec::new(),
            metrics: ParallelCompileMetrics::default(),
        });
    }

    let mut output_slots = Vec::with_capacity(modules.len());
    output_slots.resize_with(modules.len(), || None);
    let outputs: Arc<Mutex<Vec<Option<B::Output>>>> = Arc::new(Mutex::new(output_slots));

    let mut record_slots = Vec::with_capacity(modules.len());
    record_slots.resize_with(modules.len(), || None);
    let records: Arc<Mutex<Vec<Option<ModuleRecord>>>> = Arc::new(Mutex::new(record_slots));
    let error: Arc<Mutex<Option<BackendError>>> = Arc::new(Mutex::new(None));
    let options = options.clone();
    let start = Instant::now();

    let worker_count = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(modules.len())
        .max(1);
    let next_index = Arc::new(AtomicUsize::new(0));
    let worker_accumulators: Arc<Vec<Mutex<WorkerAccumulator>>> = Arc::new(
        (0..worker_count)
            .map(|_| Mutex::new(WorkerAccumulator::default()))
            .collect(),
    );

    std::thread::scope(|scope| {
        for worker_index in 0..worker_count {
            let outputs = Arc::clone(&outputs);
            let records = Arc::clone(&records);
            let error = Arc::clone(&error);
            let options = options.clone();
            let next_index = Arc::clone(&next_index);
            let worker_accumulators = Arc::clone(&worker_accumulators);
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
                    let dispatch_offset = start.elapsed();
                    let module_start = Instant::now();
                    match backend.compile(module, &options) {
                        Ok(result) => {
                            let duration = module_start.elapsed();
                            records.lock().unwrap()[index] = Some(ModuleRecord {
                                worker_index,
                                duration,
                                start_offset: dispatch_offset,
                            });
                            outputs.lock().unwrap()[index] = Some(result);
                            let mut worker_guard = worker_accumulators[worker_index]
                                .lock()
                                .expect("worker metrics poisoned");
                            worker_guard.processed_modules += 1;
                            worker_guard.busy_duration += duration;
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

    let record_guard = records.lock().unwrap();
    let module_metrics = modules
        .iter()
        .enumerate()
        .map(|(index, module)| ModuleCompileMetrics {
            module: if module.name.is_empty() {
                "<root>".to_string()
            } else {
                module.name.join("::")
            },
            duration: record_guard[index]
                .as_ref()
                .map(|record| record.duration)
                .unwrap_or_default(),
            worker_index: record_guard[index]
                .as_ref()
                .map(|record| record.worker_index)
                .unwrap_or_default(),
            start_offset: record_guard[index]
                .as_ref()
                .map(|record| record.start_offset)
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    let mut schedule = module_metrics
        .iter()
        .map(|metrics| ScheduleEntry {
            module: metrics.module.clone(),
            worker_index: metrics.worker_index,
            start_offset: metrics.start_offset,
        })
        .collect::<Vec<_>>();
    schedule.sort_by_key(|entry| entry.start_offset);

    let worker_metrics = worker_accumulators
        .iter()
        .enumerate()
        .map(|(index, accumulator)| {
            let guard = accumulator.lock().expect("worker metrics poisoned");
            WorkerMetrics {
                worker_index: index,
                processed_modules: guard.processed_modules,
                busy_duration: guard.busy_duration,
            }
        })
        .collect::<Vec<_>>();

    let metrics = ParallelCompileMetrics {
        total_duration: start.elapsed(),
        modules: module_metrics,
        worker_count,
        scheduled_modules: modules.len(),
        worker_metrics,
        schedule,
    };

    Ok(ParallelCompileReport { outputs, metrics })
}
