use super::helpers::*;
use super::*;

#[test]
fn lower_simple_function_into_ir() {
    let src = r#"
module demo

fn add(x: Int, y: Int) -> Int {
  let sum = x + y
  return sum
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    assert_eq!(ir_module.name, vec!["demo".to_string()]);
    assert_eq!(ir_module.functions.len(), 1);

    let func = &ir_module.functions[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.blocks.len(), 1);

    let block = &func.blocks[0];
    assert_eq!(block.instructions.len(), 1);
    match &block.instructions[0].kind {
        ir::InstKind::Binary { op, .. } => assert_eq!(*op, BinaryOp::Add),
        other => panic!("expected binary instruction, got {other:?}"),
    }
    match &block.terminator {
        ir::Terminator::Return(Some(ret)) => {
            assert_eq!(*ret, block.instructions[0].id);
        }
        other => panic!("expected return terminator, got {other:?}"),
    }
}

#[test]
fn lowering_literal_return_tracks_type_information() {
    let src = r#"
module demo

fn forty_two() -> Int {
  return 42
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let func = ir_module
        .functions
        .iter()
        .find(|f| f.name == "forty_two")
        .expect("function");
    assert_eq!(func.ret_type, ir::Type::Int);

    let block = &func.blocks[0];
    assert_eq!(block.instructions.len(), 1);
    assert_eq!(block.instructions[0].ty, ir::Type::Int);
    match &block.instructions[0].kind {
        ir::InstKind::Literal(Literal::Int(value)) => assert_eq!(*value, 42),
        other => panic!("expected int literal, got {other:?}"),
    }
    match &block.terminator {
        ir::Terminator::Return(Some(ret)) => {
            assert_eq!(*ret, block.instructions[0].id);
        }
        other => panic!("expected return terminator, got {other:?}"),
    }
}
