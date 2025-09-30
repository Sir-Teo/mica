use std::collections::HashSet;

use super::{Function, InstKind, Terminator, ValueId};

#[derive(Debug, Default, Clone)]
pub struct PurityReport {
    pub pure_blocks: HashSet<crate::ir::BlockId>,
    pub effectful_instructions: HashSet<ValueId>,
    pub pure_regions: Vec<Vec<crate::ir::BlockId>>,
}

impl PurityReport {
    pub fn is_block_pure(&self, id: crate::ir::BlockId) -> bool {
        self.pure_blocks.contains(&id)
    }

    pub fn is_instruction_effectful(&self, id: ValueId) -> bool {
        self.effectful_instructions.contains(&id)
    }

    pub fn regions(&self) -> &[Vec<crate::ir::BlockId>] {
        &self.pure_regions
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

    let mut regions = Vec::new();
    let mut current_region: Vec<crate::ir::BlockId> = Vec::new();
    for block in &function.blocks {
        if pure_blocks.contains(&block.id) {
            current_region.push(block.id);
        } else if !current_region.is_empty() {
            regions.push(current_region);
            current_region = Vec::new();
        }
    }
    if !current_region.is_empty() {
        regions.push(current_region);
    }

    PurityReport {
        pure_blocks,
        effectful_instructions: effectful_insts,
        pure_regions: regions,
    }
}

fn is_pure_terminator(term: &Terminator) -> bool {
    match term {
        Terminator::Return(_) | Terminator::Branch { .. } | Terminator::Jump(_) => true,
    }
}
