use std::fmt::{self, Debug};

use crate::base::OngoingCodegen;
use crate::crepr::Representable;
use crate::function::CFunction;
use crate::stmt::{handle_stmt, Statement};
use rustc_middle::mir::BasicBlockData;

#[derive(Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self { statements: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn push(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub fn clear(&mut self) {
        self.statements.clear();
    }
}

impl Representable for BasicBlock {
    fn repr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _context: &crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        for stmt in &self.statements {
            stmt.repr(f, _context)?;
        }
        Ok(())
    }
}

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

pub fn handle_bbs<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    mir: &rustc_middle::mir::Body<'tcx>,
    c_fn: &mut CFunction,
) {
    let blocks = &mir.basic_blocks;
    for (_last_bb_id, block_data) in blocks.into_iter().enumerate() {
        let block_data: &BasicBlockData = block_data;

        let statements: &Vec<rustc_middle::mir::Statement<'_>> = &block_data.statements;

        let mut n_bb = BasicBlock::new();

        // Print basic block for debugging. TODO should probably depend on a cli argument.
        n_bb.push(Statement::from_comment(format!("Basic Block: {:?}", block_data)));

        for stmt in statements {
            n_bb.push(handle_stmt(tcx, ongoing_codegen, stmt));
        }

        c_fn.push_bb(n_bb);
    }
}
