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
    assert!(func.effect_row.is_empty());
    assert_eq!(ir_module.type_of(func.params[0].ty), &ir::Type::Int);
    assert_eq!(ir_module.type_of(func.params[1].ty), &ir::Type::Int);
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
    assert_eq!(ir_module.type_of(func.ret_type), &ir::Type::Int);

    let block = &func.blocks[0];
    assert_eq!(block.instructions.len(), 1);
    assert_eq!(ir_module.type_of(block.instructions[0].ty), &ir::Type::Int);
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

    let effect_names: Vec<_> = func
        .effect_row
        .iter()
        .map(|id| ir_module.effect_name(*id).to_string())
        .collect();
    assert_eq!(effect_names, vec!["io".to_string()]);
    assert_eq!(func.params.len(), 2);
    assert_eq!(ir_module.type_of(func.params[0].ty), &ir::Type::Int);
    assert_eq!(
        ir_module.type_of(func.params[1].ty),
        &ir::Type::Named("Data".to_string())
    );
    assert_eq!(ir_module.type_of(func.ret_type), &ir::Type::Unit);

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
            assert_eq!(ir_module.type_of(ret_inst.ty), &ir::Type::Unit);
        }
        ir::Terminator::Return(None) => panic!("expected explicit unit return"),
        other => panic!("unexpected terminator: {other:?}"),
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

    assert_eq!(ir_module.type_of(func.ret_type), &ir::Type::Int);
    assert!(func.effect_row.is_empty());
    assert_eq!(func.params.len(), 1);
    assert_eq!(ir_module.type_of(func.params[0].ty), &ir::Type::Int);

    let entry_block = &func.blocks[0];
    assert!(entry_block.instructions.is_empty());
    match &entry_block.terminator {
        ir::Terminator::Return(Some(value)) => {
            assert_eq!(*value, func.params[0].value);
        }
        other => panic!("expected tail expression return, got {other:?}"),
    }
}

#[test]
fn lowering_if_expression_produces_branches_and_phi() {
    let src = r#"
module demo

fn pick(flag: Bool) -> Int {
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

    let func = ir_module
        .functions
        .iter()
        .find(|f| f.name == "pick")
        .expect("function");

    assert_eq!(
        func.blocks.len(),
        4,
        "expected entry, then, else, merge blocks"
    );

    let entry = &func.blocks[0];
    match &entry.terminator {
        ir::Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => {
            assert_eq!(*condition, func.params[0].value);
            assert_eq!(then_block.index(), 1);
            assert_eq!(else_block.index(), 2);
        }
        other => panic!("expected branch terminator, got {other:?}"),
    }

    let then_block = &func.blocks[1];
    assert_eq!(then_block.id.index(), 1);
    assert_eq!(then_block.instructions.len(), 1);
    match &then_block.terminator {
        ir::Terminator::Jump(target) => assert_eq!(target.index(), 3),
        other => panic!("expected jump terminator for then block, got {other:?}"),
    }

    let else_block = &func.blocks[2];
    assert_eq!(else_block.instructions.len(), 1);
    match &else_block.terminator {
        ir::Terminator::Jump(target) => assert_eq!(target.index(), 3),
        other => panic!("expected jump terminator for else block, got {other:?}"),
    }

    let merge_block = &func.blocks[3];
    assert_eq!(merge_block.instructions.len(), 1);
    match &merge_block.instructions[0].kind {
        ir::InstKind::Phi { incomings } => {
            assert_eq!(incomings.len(), 2);
            let mut blocks: Vec<_> = incomings.iter().map(|(block, _)| block.index()).collect();
            blocks.sort_unstable();
            assert_eq!(blocks, vec![1, 2]);
        }
        other => panic!("expected phi instruction, got {other:?}"),
    }
    match &merge_block.terminator {
        ir::Terminator::Return(Some(value)) => {
            assert_eq!(*value, merge_block.instructions[0].id);
            assert_eq!(
                ir_module.type_of(merge_block.instructions[0].ty),
                &ir::Type::Int
            );
        }
        other => panic!("expected return terminator in merge block, got {other:?}"),
    }
}
