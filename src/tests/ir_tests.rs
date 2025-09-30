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
    match ir_module.type_of(func.params[1].ty) {
        ir::Type::Record(record) => {
            assert_eq!(record.name.as_deref(), Some("Data"));
            assert_eq!(record.fields.len(), 1);
            assert_eq!(record.fields[0].name, "value");
        }
        other => panic!("expected record type for Data param, got {other:?}"),
    }
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
fn record_layout_tracks_offsets_and_size() {
    let src = r#"
module demo

type Point = { x: Int, y: Int, flag: Bool }

fn make(flag: Bool) -> Point {
  Point { x: 1, y: 2, flag }
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let point_id = ir_module
        .types
        .lookup_named("Point")
        .expect("Point type registered");
    match ir_module.type_of(point_id) {
        ir::Type::Record(record) => {
            assert_eq!(record.name.as_deref(), Some("Point"));
            assert_eq!(record.fields.len(), 3);
            assert_eq!(record.fields[0].name, "x");
            assert_eq!(record.fields[0].offset, 0);
            assert_eq!(record.fields[1].name, "y");
            assert_eq!(record.fields[1].offset, 8);
            assert_eq!(record.fields[2].name, "flag");
            assert_eq!(record.fields[2].offset, 16);
            assert_eq!(record.size, 24);
            assert_eq!(record.align, 8);
        }
        other => panic!("expected record type, got {other:?}"),
    }
}

#[test]
fn call_instructions_capture_effect_metadata() {
    let src = r#"
module demo

fn helper(io: IO) -> Int !{io} {
  1
}

fn main(io: IO) -> Int !{io} {
  helper(io)
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let main_fn = ir_module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main function present");
    let call_inst = main_fn
        .blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .find(|inst| matches!(inst.kind, ir::InstKind::Call { .. }))
        .expect("call instruction lowered");
    assert!(
        !call_inst.effects.is_empty(),
        "expected call to capture effect metadata"
    );
    let effect_names: Vec<_> = call_inst
        .effects
        .iter()
        .map(|id| ir_module.effect_name(*id).to_string())
        .collect();
    assert_eq!(effect_names, vec!["io".to_string()]);
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

#[test]
fn purity_analysis_identifies_pure_blocks() {
    let src = r#"
module demo

fn forty_two() -> Int {
  42
}

fn helper(io: IO) -> Int !{io} {
  1
}

fn main(io: IO) -> Int !{io} {
  helper(io)
}
"#;

    let module = parse(src);
    let hir = lower::lower_module(&module);
    let ir_module = ir::lower_module(&hir);

    let pure_fn = ir_module
        .functions
        .iter()
        .find(|f| f.name == "forty_two")
        .expect("pure function present");
    let report = ir::analysis::analyze_function_purity(pure_fn);
    let entry = pure_fn.blocks[0].id;
    assert!(report.is_block_pure(entry));
    assert!(report.effectful_instructions.is_empty());

    let impure_fn = ir_module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main function present");
    let report = ir::analysis::analyze_function_purity(impure_fn);
    let entry = impure_fn.blocks[0].id;
    assert!(
        !report.is_block_pure(entry),
        "call with effects should mark block impure"
    );
    assert!(report.effectful_instructions.iter().any(|id| {
        impure_fn
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .any(|inst| inst.id == *id && matches!(inst.kind, ir::InstKind::Call { .. }))
    }));
}
