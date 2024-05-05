use std::fmt::{self, Debug};

use crate::base::OngoingCodegen;
use crate::crepr::{indent, Expression, Representable, UnaryOpType};
use crate::function::CFunction;
use crate::stmt::{handle_operand, handle_stmt, handle_constant, Statement};
use crate::utils::scalar_to_u128;
use rustc_middle::mir::BasicBlockData;
use rustc_middle::mir::Operand;
use rustc_middle::mir::TerminatorKind;
use rustc_span::source_map::Spanned;
use tracing::{debug, debug_span, warn};

#[derive(Clone, PartialEq, Eq)]
pub struct BasicBlockIdentifier(pub usize);

impl Representable for BasicBlockIdentifier {
    fn repr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _context: &crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl Debug for BasicBlockIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub bb_id: BasicBlockIdentifier,
    pub statements: Vec<Statement>,
}

impl BasicBlock {
    pub fn new(bb_id: BasicBlockIdentifier) -> Self {
        Self { bb_id, statements: Vec::new() }
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
        context: &crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        indent(f, context)?;
        self.bb_id.repr(f, context)?;
        write!(f, ": {{")?;

        if context.include_newline {
            write!(f, "\n")?;
        }

        for stmt in &self.statements {
            stmt.indented_repr(f, context)?;
        }

        indent(f, context)?;
        write!(f, "}}")?;

        if context.include_newline {
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

fn handle_function_call<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    func: Operand<'tcx>,
    args: Vec<Spanned<Operand<'tcx>>>,
    destination: rustc_middle::mir::Place<'tcx>,
) -> Statement {
    let destination = Expression::Variable { local: destination.local.as_usize() }; // TODO this is probably wrong, we should handle places properly    

    match func {
        Operand::Constant(ref constant) => {
            warn!("Unimplemented function call: {:?}", func);

            let fn_call = Expression::FnCall {
                function: Box::new(handle_constant( tcx, ongoing_codegen, &constant)),
                args: args.iter().map(|arg| handle_operand(tcx, ongoing_codegen, &arg.node)).collect(),
            };

            return Statement::from_expression(Expression::Assignment {
                lhs: Box::new(destination),
                rhs: Box::new(fn_call),
            });
        }

        _ => {
            //TODO operand handling probably has to be changed to allow for functions as operands
            warn!("Unimplemented function call: {:?}", func);

            let stmt = Statement::from_comment(format!("Unimplemented function call: {:?}", func));

            return stmt;
        }
    }

    // match func

    // let call_expression = Expression::FnCall {
    //     name: func.
    //     args: args.iter().map(|arg| handle_operand(tcx, ongoing_codegen, arg)).collect(),
    // };

    // // NOTE: I'm not sure we're handling places correctly, especially when the place isn't just a normal variable, this has to be analyzed and replaced with some handle_place down the line
    // let target_variable = Expression::Variable { local: destination.local.as_usize() };

    // return Statement::from_expression(Expression::Assignment { lhs: Box::new(target_variable), rhs: Box::new(call_expression) });
}

pub fn handle_terminator<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    terminator: &rustc_middle::mir::Terminator<'tcx>,
    bb_id: &BasicBlockIdentifier,
) -> Vec<Statement> {
    let span = debug_span!("handle_terminator").entered();

    match terminator.kind.clone() {
        TerminatorKind::Call {
            func,
            args,
            destination,
            target,
            unwind: _,
            call_source: _,
            fn_span: _,
        } => {
            let fn_call = handle_function_call(tcx, ongoing_codegen, func, args, destination);

            if let Some(target) = target {
                let stmt = Statement::from_expression(Expression::Goto {
                    target: BasicBlockIdentifier(target.as_usize()),
                });

                return vec![fn_call, stmt];
            } else {
                return vec![fn_call];
            }
        }

        TerminatorKind::Return => {
            let stmt = Statement::from_expression(Expression::Return {
                value: Box::new(Expression::Variable { local: 0 }), // hardcoded 0 value, since return values are always index 0
            });

            return vec![stmt];
        }

        TerminatorKind::SwitchInt { discr, targets } => {
            let stmt = Statement::from_expression(Expression::SwitchJump {
                value: Box::new(handle_operand(tcx, ongoing_codegen, &discr)),

                cases: targets
                    .iter()
                    .map(|(val, target)| {
                        (
                            Box::new(Expression::Constant { value: format!("{}", (&val)) }), // TODO replace with proper constant representation
                            BasicBlockIdentifier(target.as_usize()),
                        )
                    })
                    .collect(),

                default: BasicBlockIdentifier(targets.otherwise().as_usize()),
            });

            return vec![stmt];
        }

        TerminatorKind::Unreachable => {
            let stmt = Statement::from_comment("Unreachable".to_string());
            return vec![stmt];
        }

        TerminatorKind::FalseEdge { real_target, imaginary_target } => {
            // imaginary target can't be reached, so we can ignore it
            let stmt = Statement::from_expression(Expression::Goto {
                target: BasicBlockIdentifier(real_target.as_usize()),
            });

            return vec![stmt];
        }

        TerminatorKind::FalseUnwind { real_target, unwind } => {
            // unwind can be ignored
            let stmt = Statement::from_expression(Expression::Goto {
                target: BasicBlockIdentifier(real_target.as_usize()),
            });

            return vec![stmt];
        }

        TerminatorKind::Assert { cond, expected, msg, target, unwind } => {
            /*  TODO we could ignore asserts, implement them with the assert define or allow the user to provide custom implementations to handle them.
                I personally think the latter would be best and would allow us to side-step other similar issues.
                We would have a default implementation that would use the assert macro, but the user could provide their own implementation.
            */
            let mut assert_operand = handle_operand(tcx, ongoing_codegen, &cond);
            if !expected {
                assert_operand =
                    Expression::UnaryOp { op: UnaryOpType::Not, val: Box::new(assert_operand) };
            }

            let assert_stmt = Statement::from_expression(Expression::FnCall {
                function: Box::new(Expression::Constant { value: "assert".to_string()}),
                args: vec![assert_operand],
            });

            let stmt = Statement::new(
                Expression::Goto { target: BasicBlockIdentifier(target.as_usize()) },
                format!("Assert: {:?}", cond),
            );

            return vec![assert_stmt, stmt];
        }

        TerminatorKind::Goto { target } => {
            let target_id = target.as_usize();

            let stmt = Statement::from_expression(Expression::Goto {
                target: BasicBlockIdentifier(target_id),
            });

            return vec![stmt];
        }

        _ => {
            warn!("Unimplemented terminator: {:?}", terminator.kind);
            let stmt =
                Statement::from_comment(format!("Unimplemented terminator: {:?}", terminator.kind));

            return vec![stmt];
        }
    }
}

pub fn handle_bbs<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    mir: &rustc_middle::mir::Body<'tcx>,
    c_fn: &mut CFunction,
) {
    let blocks = &mir.basic_blocks;

    let span = debug_span!("handle_bbs").entered();

    for (bb_id, block_data) in blocks.into_iter().enumerate() {
        let block_data: &BasicBlockData = block_data;
        let bb_id = BasicBlockIdentifier(bb_id);

        let statements: &Vec<rustc_middle::mir::Statement<'_>> = &block_data.statements;

        debug!("Basic Block: {:?}", bb_id);

        let mut n_bb = BasicBlock::new(bb_id);

        // Print basic block for debugging. TODO should probably depend on a cli argument.
        n_bb.push(Statement::from_comment(format!("Basic Block: {:?}", block_data)));

        for stmt in statements {
            n_bb.push(handle_stmt(tcx, ongoing_codegen, stmt));
        }

        let terminator_statmeents =
            handle_terminator(tcx, ongoing_codegen, &block_data.terminator(), &n_bb.bb_id);

        for stmt in terminator_statmeents {
            n_bb.push(stmt);
        }

        c_fn.push_bb(n_bb);
    }
}
