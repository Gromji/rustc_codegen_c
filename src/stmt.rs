use crate::aggregate::handle_aggregate;
use crate::crepr::{self, indent, Expression, Representable, RepresentationContext};
use crate::function::{CFunction, CodegenFunctionCx};
use crate::utils;
use rustc_middle::mir::{ConstOperand, ConstValue, Operand, Place, Rvalue, StatementKind};
use rustc_middle::ty::{ParamEnv, Ty};
use std::fmt::{self, Debug};
use tracing::{debug, debug_span, warn};

#[derive(Clone, PartialEq, Eq)]
pub struct Statement {
    pub expression: Option<Expression>,
    pub comment: Option<String>,
}

impl Statement {
    pub fn new(expression: Expression, comment: String) -> Self {
        Self { expression: Some(expression), comment: (Some(comment)) }
    }

    pub fn from_expression(expression: Expression) -> Self {
        Self { expression: Some(expression), comment: None }
    }

    pub fn from_comment(comment: String) -> Self {
        Self { expression: None, comment: Some(comment) }
    }
}

impl Representable for Statement {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
        if context.include_comments {
            if let Some(comment) = &self.comment {
                indent(f, context)?;
                write!(f, "/* {} */\n", comment)?;
            }
        }

        if let Some(expression) = &self.expression {
            indent(f, context)?;
            expression.repr(f, context)?;
            write!(f, ";")?;
            if context.include_newline {
                write!(f, "\n")?;
            }
        }

        Ok(())
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

pub fn handle_stmt<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    stmt: &'tcx rustc_middle::mir::Statement<'tcx>,
) -> Statement {
    let span = debug_span!("handle_stmt").entered();

    debug!("Statement: {:?}", stmt);
    debug!("Kind: {:?}", stmt.kind);

    let expression = match &stmt.kind {
        StatementKind::Assign(val) => handle_assign(fn_cx, c_fn, &val.0, &val.1),

        _ => crepr::Expression::NoOp {},
    };

    let statement = Statement::new(expression, format!("//{:?}", stmt).into());

    span.exit();

    return statement;
}

pub fn handle_operand<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    operand: &Operand<'tcx>,
) -> crepr::Expression {
    match operand {
        Operand::Copy(place) => Expression::Variable { local: place.local.as_usize(), idx: None },
        // move operations can be treated as a copy operation (I think)
        Operand::Move(place) => Expression::Variable { local: place.local.as_usize(), idx: None },

        Operand::Constant(constant) => handle_constant(fn_cx, constant),
    }
}

fn handle_assign<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    place: &Place<'tcx>,
    rvalue: &'tcx Rvalue<'tcx>,
) -> crepr::Expression {
    let span = debug_span!("handle_assign").entered();
    debug!("place( {:?} )", place);
    debug!("rvalue( {:?} )", rvalue);

    let expression = match rvalue {
        Rvalue::Use(operand) => handle_operand(fn_cx, operand),

        Rvalue::BinaryOp(op, operands) => {
            let lhs = handle_operand(fn_cx, &operands.0);
            let rhs = handle_operand(fn_cx, &operands.1);

            crepr::Expression::BinaryOp { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) }
        }

        Rvalue::CheckedBinaryOp(op, operands) => {
            let lhs = handle_operand(fn_cx, &operands.0);
            let rhs = handle_operand(fn_cx, &operands.1);

            crepr::Expression::CheckedBinaryOp {
                op: op.into(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        Rvalue::Aggregate(kind, fields) => {
            // Return instantly because it already handles assignments.
            return handle_aggregate(fn_cx, c_fn, place, kind, fields.iter());
        }

        _ => {
            warn!("Unhandled rvalue: {:?}", rvalue);
            crepr::Expression::NoOp {}
        }
    };

    span.exit();

    return crepr::Expression::Assignment {
        lhs: Box::new(crepr::Expression::Variable { local: place.local.as_usize(), idx: None }),
        rhs: Box::new(expression),
    };
}

pub fn handle_constant<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    const_op: &ConstOperand<'tcx>,
) -> Expression {
    let constant = &const_op.const_;

    fn_cx.monomorphize(constant.ty());
    let value = constant
        .eval(fn_cx.tcx, ParamEnv::reveal_all(), const_op.span)
        .expect("Constant evaluation failed");

    handle_const_value(&value, &constant.ty())
}

fn handle_const_value<'tcx>(val: &ConstValue, ty: &Ty) -> Expression {
    match val {
        rustc_middle::mir::ConstValue::Scalar(scalar) => match scalar {
            rustc_const_eval::interpret::Scalar::Int(i) => {
                return Expression::Constant {
                    value: format!("{}", utils::scalar_to_u128(&scalar)),
                }; // todo handle this better
            }

            rustc_const_eval::interpret::Scalar::Ptr(_, _) => todo!("Ptr"),
        },

        rustc_middle::mir::ConstValue::ZeroSized => {
            debug!("Zerosized kind {:?}, val {:?}", ty.kind(), ty);
            return Expression::Constant { value: ty.to_string() };
        }
        _ => {
            warn!("Unhandled constant: {:?}", val);
            return Expression::NoOp {};
        }
    }
}
