use std::collections::HashSet;

use super::{Function, InstKind, Terminator, ValueId};

#[derive(Debug, Default, Clone)]
pub struct PurityReport {
    pub pure_blocks: HashSet<crate::ir::BlockId>,
    pub effectful_instructions: HashSet<ValueId>,
}

impl PurityReport {
    pub fn is_block_pure(&self, id: crate::ir::BlockId) -> bool {
        self.pure_blocks.contains(&id)
    }

    pub fn is_instruction_effectful(&self, id: ValueId) -> bool {
        self.effectful_instructions.contains(&id)
    }
}

pub fn analyze_function_purity(function: &Function) -> PurityReport {
    let mut pure_blocks = HashSet::new();
    let mut effectful_insts = HashSet::new();

    for block in &function.blocks {
        let mut block_pure = true;
        for inst in &block.instructions {
            let mut effectful = !inst.effects.is_empty();
            if matches!(inst.kind, InstKind::Call { .. }) && inst.effects.is_empty() {
                // conservatively assume external calls without metadata are effectful
                effectful = true;
            }
            if effectful {
                block_pure = false;
                effectful_insts.insert(inst.id);
            }
        }

        if block_pure && is_pure_terminator(&block.terminator) {
            pure_blocks.insert(block.id);
        }
    }

    PurityReport {
        pure_blocks,
        effectful_instructions: effectful_insts,
    }
}

fn is_pure_terminator(term: &Terminator) -> bool {
    match term {
        Terminator::Return(_) | Terminator::Branch { .. } | Terminator::Jump(_) => true,
    }
}
