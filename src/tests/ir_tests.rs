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
    assert_eq!(func.effect_row, Vec::<String>::new());
    assert_eq!(func.params[0].ty, ir::Type::Int);
    assert_eq!(func.params[1].ty, ir::Type::Int);
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

#[test]
fn lowering_preserves_effect_rows_and_declared_types() {
    let src = r#"
module demo

type Data = { value: Int }

fn process(count: Int, data: Data) -> Unit !{io} {
  let _tmp = data
  return ()
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let func = ir_module
        .functions
        .iter()
        .find(|f| f.name == "process")
        .expect("function");

    assert_eq!(func.effect_row, vec!["io".to_string()]);
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].ty, ir::Type::Int);
    assert_eq!(func.params[1].ty, ir::Type::Named("Data".to_string()));
    assert_eq!(func.ret_type, ir::Type::Unit);

    let entry_block = func
        .blocks
        .iter()
        .find(|block| block.id.index() == 0)
        .expect("entry block");
    match &entry_block.terminator {
        ir::Terminator::Return(Some(value)) => {
            let ret_inst = entry_block
                .instructions
                .iter()
                .find(|inst| inst.id == *value)
                .expect("return instruction");
            assert_eq!(ret_inst.ty, ir::Type::Unit);
        }
        ir::Terminator::Return(None) => panic!("expected explicit unit return"),
    }
}

#[test]
fn lowering_treats_tail_expression_as_return() {
    let src = r#"
module demo

fn identity(x: Int) -> Int {
  x
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let func = ir_module
        .functions
        .iter()
        .find(|f| f.name == "identity")
        .expect("function");

    assert_eq!(func.ret_type, ir::Type::Int);
    assert!(func.effect_row.is_empty());
    assert_eq!(func.params.len(), 1);
    assert_eq!(func.params[0].ty, ir::Type::Int);

    let entry_block = &func.blocks[0];
    assert!(entry_block.instructions.is_empty());
    match &entry_block.terminator {
        ir::Terminator::Return(Some(value)) => {
            assert_eq!(*value, func.params[0].value);
        }
        other => panic!("expected tail expression return, got {other:?}"),
    }
}
