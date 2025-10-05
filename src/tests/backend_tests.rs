use super::helpers::*;
use super::*;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn text_backend_renders_effect_rows_and_types() {
    let src = r#"
module backend.sample

fn transform(x: Int, io: IO) -> Int !{io} {
  let bumped = x + 1
  return bumped
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let text_backend = backend::text::TextBackend;
    let output = backend::run(
        &text_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    assert!(output.contains("module backend.sample"));
    assert!(output.contains("fn transform(x: Int, io: IO) -> Int !{io}"));
    assert!(output.contains("block 0:"));
    assert!(output.contains("return %"));
    assert!(
        output.contains("+ %") || output.contains("call"),
        "expected arithmetic or call instruction in block: {}",
        output
    );
}

#[test]
fn text_backend_formats_records_and_unknown_types() {
    let src = r#"
module backend.records

type Data = { value: Int }

fn build(flag: Bool) {
  let record = Data { value: 1 }
  if flag {
    record
  } else {
    record
  }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let text_backend = backend::text::TextBackend;
    let output = backend::run(
        &text_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    assert!(output.contains("record Data"));
    assert!(output.contains("fn build(flag: Bool)"));
    assert!(output.contains("return %"));
    assert!(
        output.contains("branch %"),
        "expected conditional branch in text backend output: {}",
        output
    );
    assert!(
        output.contains("phi"),
        "expected phi node in merge block output: {}",
        output
    );
}

#[test]
fn llvm_backend_renders_integer_arithmetic() {
    let src = r#"
module backend.llvm

fn transform(x: Int, io: IO) -> Int !{io} {
  let bumped = x + 1
  return bumped
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("target datalayout"));
    assert!(ir.contains("define i64 @transform(i64 %x, ptr %io)"));
    assert!(ir.contains("; effects: io"));
    assert!(ir.contains("bb0:"));
    assert!(ir.contains("%2 = add i64 0, 1"));
    assert!(ir.contains("%3 = add i64 %0, %2"));
    assert!(ir.contains("ret i64 %3"));
}

#[test]
fn llvm_backend_handles_branch_and_phi() {
    let src = r#"
module backend.branch

fn choose(flag: Bool) -> Int {
  if flag {
    1
  } else {
    2
  }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("br i1 %0, label %bb1, label %bb2"));
    assert!(ir.contains("%3 = phi i64 [ %1, %bb1 ], [ %2, %bb2 ]"));
    assert!(ir.contains("ret i64 %3"));
}

#[test]
fn llvm_backend_interns_string_literals() {
    let src = r#"
module backend.strings

fn message() -> String {
  "hi"
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("@.str0 = private constant [3 x i8] c\"hi\\00\""));
    assert!(ir.contains("%0 = getelementptr inbounds ([3 x i8], ptr @.str0, i32 0, i32 0)"));
    assert!(ir.contains("ret ptr %0"));
}

#[test]
fn llvm_backend_returns_void_for_unit() {
    let src = r#"
module backend.unit

fn noop() {
  ()
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("define void @noop()"));
    assert!(ir.contains("bb0:"));
    assert!(ir.contains("ret void"));
}

#[test]
fn llvm_backend_emits_record_stub_call() {
    let src = r#"
module backend.record

type Data = { value: Int }

fn build() -> Data {
  Data { value: 1 }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("%record.Data = type { i64 }"));
    assert!(ir.contains("define %record.Data @build()"));
    assert!(ir.contains("insertvalue %record.Data"));
    assert!(ir.contains("ret %record.Data %"));
}

#[test]
fn llvm_backend_preserves_call_return_types() {
    let src = r#"
module backend.call_types

type Vec2 = { x: Int, y: Int }

fn build_vec2(x: Int, y: Int) -> Vec2 {
  Vec2 { x, y }
}

fn use_call() -> Vec2 {
  build_vec2(1, 2)
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect("backend output");

    let ir = output.as_str();
    assert!(ir.contains("define %record.Vec2 @use_call()"));
    assert!(ir.contains("call %record.Vec2 @build_vec2"));
    assert!(ir.contains("ret %record.Vec2 %"));
}

#[test]
fn llvm_backend_rejects_incomplete_record_literals() {
    let src = r#"
module backend.record_error

type Data = { value: Int, flag: Bool }

fn build() -> Data {
  Data { value: 1 }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let err = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect_err("expected backend failure");

    match err {
        backend::BackendError::Unsupported(message) => {
            assert!(
                message.contains("missing field") || message.contains("record literal"),
                "unexpected message: {}",
                message
            );
        }
        other => panic!("expected unsupported error, got {other:?}"),
    }
}

#[test]
fn llvm_backend_rejects_untyped_record_literals() {
    let src = r#"
module backend.record_untyped

fn make() {
  Missing { flag: true }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let llvm_backend = backend::llvm::LlvmBackend::default();
    let err = backend::run(
        &llvm_backend,
        &ir_module,
        &backend::BackendOptions::default(),
    )
    .expect_err("expected backend failure");

    match err {
        backend::BackendError::Unsupported(message) => {
            assert!(
                message.contains("record literal"),
                "unexpected message: {}",
                message
            );
        }
        other => panic!("expected unsupported error, got {other:?}"),
    }
}

#[test]
fn native_backend_builds_and_runs_main() {
    let src = r#"
module backend.native_run

fn main() -> Int {
  let base = 40
  let answer = base + 2
  answer - 42
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
    let artifact = backend::run(&backend, &ir_module, &backend::BackendOptions::default())
        .expect("native backend artifact");

    let mut exe_path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    exe_path.push(format!("mica-native-test-{nanos}"));

    artifact
        .link_executable(&exe_path)
        .expect("link executable");

    let status = Command::new(&exe_path).status().expect("execute binary");
    assert!(status.success(), "process exit: {status}");
    fs::remove_file(&exe_path).ok();
}

#[test]
fn native_backend_handles_unit_functions() {
    let src = r#"
module backend.native_unit

fn noop() {
  ()
}

fn caller() {
  noop()
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
    let artifact = backend::run(&backend, &ir_module, &backend::BackendOptions::default())
        .expect("native backend artifact");

    assert!(
        artifact.c_source.contains("void noop(void)"),
        "expected void signature for noop: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("void caller(void)"),
        "expected void signature for caller: {}",
        artifact.c_source
    );
    assert!(
        !artifact.c_source.contains("return 0;"),
        "unit functions should not return numeric zero: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("noop();\n  int64_t v"),
        "unit calls should lower to call plus stub value: {}",
        artifact.c_source
    );
    assert!(
        !artifact.c_source.contains("return v"),
        "unit functions should not return temporary values: {}",
        artifact.c_source
    );
}

#[derive(Clone)]
struct CountingBackend {
    counter: Arc<AtomicUsize>,
    delay: Duration,
}

impl CountingBackend {
    fn new(delay: Duration) -> Self {
        CountingBackend {
            counter: Arc::new(AtomicUsize::new(0)),
            delay,
        }
    }
}

impl backend::Backend for CountingBackend {
    type Output = usize;

    fn compile(
        &self,
        _module: &ir::Module,
        _options: &backend::BackendOptions,
    ) -> backend::BackendResult<Self::Output> {
        let index = self.counter.fetch_add(1, Ordering::SeqCst);
        if !self.delay.is_zero() {
            std::thread::sleep(self.delay);
        }
        Ok(index)
    }
}

#[test]
fn parallel_backend_reports_worker_metrics() {
    let backend = CountingBackend::new(Duration::from_millis(1));
    let modules = ["alpha", "beta", "gamma", "delta"]
        .iter()
        .map(|name| {
            let src = format!("module parallel.{name}\n\nfn entry() -> Int {{ 1 }}\n");
            let module = parse(&src);
            let hir = lower::lower_module(&module);
            ir::lower_module(&hir)
        })
        .collect::<Vec<_>>();

    let report = backend::run_parallel(&backend, &modules, &backend::BackendOptions::default())
        .expect("parallel report");

    assert_eq!(report.outputs.len(), modules.len());
    assert_eq!(
        report.metrics.worker_metrics.len(),
        report.metrics.worker_count
    );
    let processed: usize = report
        .metrics
        .worker_metrics
        .iter()
        .map(|metrics| metrics.processed_modules)
        .sum();
    assert_eq!(processed, modules.len());
    assert!(
        report
            .metrics
            .schedule
            .windows(2)
            .all(|pair| pair[0].start_offset <= pair[1].start_offset)
    );
    for entry in &report.metrics.schedule {
        assert!(entry.worker_index < report.metrics.worker_count);
    }
}

#[test]
fn native_backend_threads_runtime_capabilities() {
    let src = r#"
module backend.native_capabilities

fn log(io: IO) !{io} {
  io.println("hello")
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
    let artifact = backend::run(&backend, &ir_module, &backend::BackendOptions::default())
        .expect("native backend artifact");

    assert!(
        artifact
            .c_source
            .contains("mica_runtime_require_capability(\"io\")"),
        "runtime guards missing from C source: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("mica_runtime_io_write_line"),
        "expected IO runtime shim in C source: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("MICA_RUNTIME_CAPABILITY_NAMES"),
        "capability registry not emitted: {}",
        artifact.c_source
    );
}

#[test]
fn recommended_worker_count_reserves_headroom() {
    assert_eq!(backend::recommended_worker_count_with_available(1, 8), 1);
    assert_eq!(backend::recommended_worker_count_with_available(2, 8), 1);
    assert_eq!(backend::recommended_worker_count_with_available(3, 8), 2);
    assert_eq!(backend::recommended_worker_count_with_available(8, 8), 7);
    assert_eq!(backend::recommended_worker_count_with_available(4, 2), 2);
    assert_eq!(backend::recommended_worker_count_with_available(4, 1), 1);
}

#[test]
fn native_backend_emits_record_literals() {
    let src = r#"
module backend.native_record

type Vec2 = { x: Int, y: Int }

fn build(x: Int, y: Int) -> Vec2 {
  Vec2 { x, y }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
    let artifact = backend::run(&backend, &ir_module, &backend::BackendOptions::default())
        .expect("native backend artifact");

    assert!(
        artifact.c_source.contains("typedef struct record_Vec2"),
        "expected record typedef in native output: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("record_Vec2 build"),
        "expected Vec2 signature in native output: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains("(record_Vec2){ .x ="),
        "expected record literal initializer in native output: {}",
        artifact.c_source
    );
    assert!(
        artifact.c_source.contains(".y ="),
        "expected y field initialization in native output: {}",
        artifact.c_source
    );
}

#[test]
fn parallel_backend_reports_metrics_for_many_modules() {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone)]
    struct RecordingBackend {
        delay: Duration,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl backend::Backend for RecordingBackend {
        type Output = String;

        fn compile(
            &self,
            module: &ir::Module,
            _options: &backend::BackendOptions,
        ) -> backend::BackendResult<Self::Output> {
            std::thread::sleep(self.delay);
            let name = if module.name.is_empty() {
                "<root>".to_string()
            } else {
                module.name.join("::")
            };
            self.log.lock().unwrap().push(name.clone());
            Ok(name)
        }
    }

    let delay = Duration::from_millis(5);
    let backend = RecordingBackend {
        delay,
        log: Arc::new(Mutex::new(Vec::new())),
    };

    let sources = (0..8)
        .map(|index| format!("module backend.parallel{index}\n\nfn value() -> Int {{ {index} }}\n"))
        .collect::<Vec<_>>();

    let modules = sources
        .iter()
        .map(|src| {
            let module = parse(src);
            let hir = lower::lower_module(&module);
            ir::lower_module(&hir)
        })
        .collect::<Vec<_>>();

    let report = backend::run_parallel(&backend, &modules, &backend::BackendOptions::default())
        .expect("parallel backend report");

    assert_eq!(report.outputs.len(), modules.len());
    assert_eq!(report.metrics.modules.len(), modules.len());
    
    // Use the actual recommended_worker_count logic which reserves headroom
    let available = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let expected_workers = backend::recommended_worker_count_with_available(modules.len(), available);
    assert_eq!(report.metrics.worker_count, expected_workers);
    assert_eq!(report.metrics.scheduled_modules, modules.len());
    assert!(report.metrics.total_duration >= delay);
    assert!(
        report
            .metrics
            .modules
            .iter()
            .all(|metric| metric.duration >= Duration::from_millis(1))
    );

    let expected_names = modules
        .iter()
        .map(|module| {
            if module.name.is_empty() {
                "<root>".to_string()
            } else {
                module.name.join("::")
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(report.outputs, expected_names);
    let metric_names = report
        .metrics
        .modules
        .iter()
        .map(|metric| metric.module.clone())
        .collect::<Vec<_>>();
    assert_eq!(metric_names, expected_names);

    let log = backend.log.lock().unwrap().clone();
    assert_eq!(log.len(), modules.len());
    for name in expected_names {
        assert!(log.contains(&name));
    }
}
