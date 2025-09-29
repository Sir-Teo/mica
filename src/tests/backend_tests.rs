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
}
