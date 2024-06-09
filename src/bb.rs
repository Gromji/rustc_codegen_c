use std::fmt::{self, Debug};

use crate::crepr::{indent, Representable};
use crate::expression::{Expression, UnaryOpType, VariableAccess};
use crate::fatptr::{FAT_PTR_DATA_FIELD, FAT_PTR_META_FIELD};
use crate::function::{format_fn_name, CFunction, CodegenFunctionCx};
use crate::stmt::{
    handle_operand, handle_operand_with_access, handle_place, handle_stmt, Statement,
};
use crate::ty::CType;
use rustc_middle::mir::BasicBlockData;
use rustc_middle::mir::Operand;
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::{InstanceDef, ParamEnv, TyKind};
use rustc_span::source_map::Spanned;
use tracing::{debug, debug_span, warn};

#[derive(Clone, PartialEq, Eq)]
pub struct BasicBlockIdentifier(pub usize);

impl Representable for BasicBlockIdentifier {
    fn repr(
        &self,
        f: &mut (dyn fmt::Write),
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
        Self {
            bb_id,
            statements: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn push(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.statements.clear();
    }
}

impl Representable for BasicBlock {
    fn repr(
        &self,
        f: &mut (dyn fmt::Write),
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

fn handle_dynamic_dispatch<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    args: Vec<Spanned<Operand<'tcx>>>,
    fn_type: CType,
    fn_offset: usize,
) -> Expression {
    let vtable_arg = args
        .get(0)
        .expect("dynamic dispatch call must contain at least one arg")
        .clone();

    let vtable_access = handle_operand_with_access(
        fn_cx,
        &vtable_arg.node,
        vec![
            VariableAccess::Field {
                name: FAT_PTR_META_FIELD.to_string(),
            }, 
            VariableAccess::Cast {
                ty: CType::Pointer(Box::new(CType::Pointer(Box::new(CType::Void)))),
            },
            VariableAccess::Index {
                expression: Expression::const_int(fn_offset as i128),
            },
            VariableAccess::Cast { ty: fn_type },
        ],
    );

    let self_access = handle_operand_with_access(
        fn_cx,
        &args[0].node,
        vec![VariableAccess::Field {
            name: FAT_PTR_DATA_FIELD.to_string(),
        }],
    );

    let mut fn_args: Vec<Expression> = Vec::new();
    fn_args.push(self_access);
    fn_args.extend(
        args.iter()
            .skip(1)
            .map(|arg| handle_operand(fn_cx, &arg.node)),
    );

    Expression::FnCall {
        function: Box::new(vtable_access),
        args: fn_args,
    }
}

fn handle_function_call<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    func: Operand<'tcx>,
    args: Vec<Spanned<Operand<'tcx>>>,
    destination: rustc_middle::mir::Place<'tcx>,
) -> Statement {
    let _span = debug_span!("handle_function_call").entered();
    debug!("Function call: {:?}, args {:?}", func, args);

    let destination = handle_place(fn_cx, &destination);

    match func {
        Operand::Constant(ref constant) => {
            let ty = fn_cx.monomorphize(constant.ty());

            // TODO Refactor this mess
            let fn_call = match ty.kind() {
                rustc_middle::ty::TyKind::FnDef(def_id, substs) => {
                    let instance = rustc_middle::ty::Instance::resolve(
                        fn_cx.tcx,
                        rustc_middle::ty::ParamEnv::reveal_all(),
                        *def_id,
                        substs,
                    )
                    .unwrap();
                    if let Some(instance) = instance {
                        if let InstanceDef::Virtual(_def_id, fn_offset) = instance.def {
                            let fn_ty =
                                instance.ty(fn_cx.tcx, rustc_middle::ty::ParamEnv::reveal_all());

                            let sig = match fn_ty.kind() {
                                TyKind::FnDef(_, _) => fn_ty.fn_sig(fn_cx.tcx),
                                TyKind::Closure(_, args) => args.as_closure().sig(),
                                _ => panic!("Not a function type: {:?}", fn_ty.kind()),
                            };

                            let sig = fn_cx.monomorphize(sig);
                            let sig = fn_cx
                                .tcx
                                .normalize_erasing_late_bound_regions(ParamEnv::reveal_all(), sig);

                            let fn_ctype = fn_cx.erased_fn_pointer_type(&sig);

                            handle_dynamic_dispatch(fn_cx, args, fn_ctype, fn_offset)
                        } else {
                            let fn_name_expr = Expression::Constant {
                                value: format_fn_name(&fn_cx.tcx.symbol_name(instance)),
                            };

                            Expression::FnCall {
                                function: Box::new(fn_name_expr),
                                args: args
                                    .iter()
                                    .map(|arg| handle_operand(fn_cx, &arg.node))
                                    .collect(),
                            }
                        }
                    } else {
                        panic!("Instance not found for {:?}", constant);
                    }
                }

                _ => {
                    panic!("Unimplemented function call: {:?}", func);
                }
            };

            return Statement::from_expression(Expression::Assignment {
                lhs: Box::new(destination),
                rhs: Box::new(fn_call),
            });
        }

        _ => {
            //TODO operand handling probably has to be changed to allow for functions as operands
            panic!("Unimplemented function call: {:?}", func);
        }
    }
}

pub fn handle_terminator<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    terminator: &rustc_middle::mir::Terminator<'tcx>,
    _bb_id: &BasicBlockIdentifier,
) -> Vec<Statement> {
    let _span = debug_span!("handle_terminator").entered();

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
            let fn_call = handle_function_call(fn_cx, func, args, destination);

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
                value: Expression::vari(0), // hardcoded 0 value, since return values are always index 0
            });

            return vec![stmt];
        }

        TerminatorKind::SwitchInt { discr, targets } => {
            let stmt = Statement::from_expression(Expression::SwitchJump {
                value: Box::new(handle_operand(fn_cx, &discr)),

                cases: targets
                    .iter()
                    .map(|(val, target)| {
                        (
                            Box::new(Expression::Constant {
                                value: format!("{}", (&val)),
                            }), // TODO replace with proper constant representation
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

        TerminatorKind::FalseEdge { real_target, .. } => {
            // imaginary target can't be reached, so we can ignore it
            let stmt = Statement::from_expression(Expression::Goto {
                target: BasicBlockIdentifier(real_target.as_usize()),
            });

            return vec![stmt];
        }

        TerminatorKind::FalseUnwind { real_target, .. } => {
            // unwind can be ignored
            let stmt = Statement::from_expression(Expression::Goto {
                target: BasicBlockIdentifier(real_target.as_usize()),
            });

            return vec![stmt];
        }

        TerminatorKind::Assert {
            cond,
            expected,
            msg: _,
            target,
            ..
        } => {
            /*  TODO we could ignore asserts, implement them with the assert define or allow the user to provide custom implementations to handle them.
                I personally think the latter would be best and would allow us to side-step other similar issues.
                We would have a default implementation that would use the assert macro, but the user could provide their own implementation.
            */
            let mut assert_operand = handle_operand(fn_cx, &cond);
            if !expected {
                assert_operand = Expression::UnaryOp {
                    op: UnaryOpType::Not,
                    val: Box::new(assert_operand),
                };
            }

            let assert_stmt = Statement::from_expression(Expression::FnCall {
                function: Box::new(Expression::Constant {
                    value: "assert".to_string(),
                }),
                args: vec![assert_operand],
            });

            let stmt = Statement::new(
                Expression::Goto {
                    target: BasicBlockIdentifier(target.as_usize()),
                },
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

        TerminatorKind::InlineAsm {
            template,
            operands,
            options,
            line_spans,
            targets,
            unwind,
        } => {
            debug!(
                "InlineAsm: {:?}, Operands: {:?}, Options: {:?}, Line Spans: {:?}, Targets: {:?}, Unwind: {:?}",
                template, operands, options, line_spans, targets, unwind
            );

            template
                .iter()
                .flat_map(|piece| match piece {
                    rustc_ast::ast::InlineAsmTemplatePiece::String(s) => {
                        if s.trim().is_empty() {
                            return vec![];
                        }
                        vec![Statement::from_expression(Expression::InlineAsm {
                            asm: s.clone(),
                        })]
                    }

                    rustc_ast::ast::InlineAsmTemplatePiece::Placeholder {
                        operand_idx: _,
                        modifier: _,
                        span: _,
                    } => {
                        // let operand = operands[*operand_idx];

                        // let expr = handle_operand(fn_cx, &operand);

                        // vec![Statement::from_expression(Expression::InlineAsm { asm: () })]
                        panic!("Unimplemented InlineAsm Placeholder");
                    }
                })
                .collect()
        }

        _ => {
            warn!("Unimplemented terminator: {:?}", terminator.kind);
            let stmt =
                Statement::from_comment(format!("Unimplemented terminator: {:?}", terminator.kind));

            return vec![stmt];
        }
    }
}

pub fn handle_bbs<'tcx, 'ccx>(fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>, c_fn: &mut CFunction) {
    let blocks = &fn_cx.mir.basic_blocks;

    let _span = debug_span!("handle_bbs").entered();

    for (bb_id, block_data) in blocks.into_iter().enumerate() {
        let block_data: &BasicBlockData = block_data;
        let bb_id = BasicBlockIdentifier(bb_id);

        let statements: &Vec<rustc_middle::mir::Statement<'tcx>> = &block_data.statements;

        debug!("Basic Block: {:?}", bb_id);

        let mut n_bb = BasicBlock::new(bb_id);

        // Print basic block for debugging. TODO should probably depend on a cli argument.
        n_bb.push(Statement::from_comment(format!(
            "Basic Block: {:?}",
            block_data
        )));

        for stmt in statements {
            n_bb.push(handle_stmt(fn_cx, c_fn, stmt));
        }

        let terminator_statmeents = handle_terminator(fn_cx, &block_data.terminator(), &n_bb.bb_id);

        for stmt in terminator_statmeents {
            n_bb.push(stmt);
        }

        c_fn.push_bb(n_bb);
    }
}
