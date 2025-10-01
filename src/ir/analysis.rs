use std::collections::{HashMap, HashSet};

use super::{BlockId, Function, InstKind, Terminator, ValueId};

#[derive(Debug, Default, Clone)]
pub struct PurityReport {
    pub pure_blocks: HashSet<crate::ir::BlockId>,
    pub effectful_instructions: HashSet<ValueId>,
    pub pure_regions: Vec<Vec<crate::ir::BlockId>>,
    pub block_effects: HashMap<crate::ir::BlockId, BlockPurity>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPurity {
    Pure,
    Effectful,
}

pub fn analyze_function_purity(function: &Function) -> PurityReport {
    let mut pure_blocks = HashSet::new();
    let mut effectful_insts = HashSet::new();
    let mut block_effects = HashMap::new();
    let mut adjacency: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

    for block in &function.blocks {
        adjacency.entry(block.id).or_default();
    }

    for block in &function.blocks {
        let mut block_pure = true;
        for inst in &block.instructions {
            let mut effectful = !inst.effects.is_empty();
            if let InstKind::Call { func, .. } = &inst.kind {
                if inst.effects.is_empty() {
                    // Calls without effect metadata are pure only when we can prove
                    // they target another IR function. Method calls or unresolved
                    // references remain conservatively effectful so future lowering
                    // phases can attach metadata without breaking assumptions here.
                    effectful = matches!(func, super::FuncRef::Method(_));
                }
            }
            if effectful {
                block_pure = false;
                effectful_insts.insert(inst.id);
            }
        }

        if !is_pure_terminator(&block.terminator) {
            block_pure = false;
        }

        block_effects.insert(
            block.id,
            if block_pure {
                BlockPurity::Pure
            } else {
                BlockPurity::Effectful
            },
        );

        if block_pure {
            pure_blocks.insert(block.id);
        }

        match &block.terminator {
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                adjacency
                    .entry(block.id)
                    .or_default()
                    .extend([*then_block, *else_block]);
                adjacency.entry(*then_block).or_default().insert(block.id);
                adjacency.entry(*else_block).or_default().insert(block.id);
            }
            Terminator::Jump(target) => {
                adjacency.entry(block.id).or_default().insert(*target);
                adjacency.entry(*target).or_default().insert(block.id);
            }
            Terminator::Return(_) => {}
        }
    }

    let mut regions = Vec::new();
    let mut visited = HashSet::new();
    let mut sorted_pure: Vec<_> = pure_blocks.iter().copied().collect();
    sorted_pure.sort_by_key(|id| id.index());

    for block_id in sorted_pure {
        if !visited.insert(block_id) {
            continue;
        }
        let mut stack = vec![block_id];
        let mut region = Vec::new();
        while let Some(current) = stack.pop() {
            region.push(current);
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if pure_blocks.contains(neighbor) && visited.insert(*neighbor) {
                        stack.push(*neighbor);
                    }
                }
            }
        }
        region.sort_by_key(|id| id.index());
        regions.push(region);
    }

    PurityReport {
        pure_blocks,
        effectful_instructions: effectful_insts,
        pure_regions: regions,
        block_effects,
    }
}

fn is_pure_terminator(term: &Terminator) -> bool {
    match term {
        Terminator::Return(_) | Terminator::Branch { .. } | Terminator::Jump(_) => true,
    }
}
