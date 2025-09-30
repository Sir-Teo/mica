use super::helpers::*;
use super::*;

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
    let text_backend = backend::text::TextBackend::default();
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
    let text_backend = backend::text::TextBackend::default();
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
    assert!(ir.contains("define ptr @build()"));
    assert!(ir.contains("call ptr @__mica_record_stub(i64 %0)"));
    assert!(ir.contains("ret ptr %1"));
}
