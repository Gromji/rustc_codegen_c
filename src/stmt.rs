use crate::base::OngoingCodegen;
use crate::function::CFunction;
use rustc_middle::mir::{ConstOperand, Operand, Place, Rvalue, StatementKind};

use tracing::{debug, debug_span, warn};

use crate::crepr::{self, Expression};

pub fn handle_stmt<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    stmt: &rustc_middle::mir::Statement<'tcx>,
    c_fn: &mut CFunction,
) {
    let span = debug_span!("handle_stmt").entered();

    debug!("Statement: {:?}", stmt);
    debug!("Kind: {:?}", stmt.kind);

    let expression = match &stmt.kind {
        StatementKind::Assign(val) => handle_assign(tcx, ongoing_codegen, &val.0, &val.1),

        _ => crepr::Expression::NoOp {},
    };

    let statement = crepr::Statement { expression, comment: Some(format!("//{:?}", stmt).into()) };

    // we shouldn't be pushing strings directly into the function body, we should be pushing statements
    c_fn.push(format!("{:?}", statement).as_str(), true, 0);

    span.exit();
}

fn handle_operand<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    operand: &Operand<'tcx>,
) -> crepr::Expression {
    match operand {
        Operand::Copy(place) => Expression::Variable { local: place.local.as_usize() },
        // move operations can be treated as a copy operation (I think)
        Operand::Move(place) => Expression::Variable { local: place.local.as_usize() },

        Operand::Constant(constant) => handle_constant(tcx, ongoing_codegen, constant),
    }
}

fn handle_assign<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    place: &Place<'tcx>,
    rvalue: &Rvalue<'tcx>,
) -> crepr::Expression {
    let span = debug_span!("handle_assign").entered();
    debug!("place( {:?} )", place);
    debug!("rvalue( {:?} )", rvalue);

    let expression = match rvalue {
        Rvalue::Use(operand) => handle_operand(tcx, ongoing_codegen, operand),

        Rvalue::BinaryOp(op, operands) => {
            let lhs = handle_operand(tcx, ongoing_codegen, &operands.0);
            let rhs = handle_operand(tcx, ongoing_codegen, &operands.1);

            crepr::Expression::BinaryOp { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) }
        }

        Rvalue::CheckedBinaryOp(op, operands) => {
            let lhs = handle_operand(tcx, ongoing_codegen, &operands.0);
            let rhs = handle_operand(tcx, ongoing_codegen, &operands.1);

            crepr::Expression::CheckedBinaryOp {
                op: op.into(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }

        _ => {
            warn!("Unhandled rvalue: {:?}", rvalue);
            crepr::Expression::NoOp {}
        }
    };

    span.exit();

    return crepr::Expression::Assignment {
        lhs: Box::new(crepr::Expression::Variable { local: place.local.as_usize() }),
        rhs: Box::new(expression),
    };
}

fn handle_constant<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    _ongoing_codegen: &mut OngoingCodegen,
    constant: &ConstOperand<'tcx>,
) -> Expression {
    match constant.const_ {
        rustc_middle::mir::Const::Unevaluated(c, t) => {
            match tcx.const_eval_poly(c.def) {
                Ok(val) => {
                    return Expression::Constant { value: format!("{:?}", val).into() }; // todo handle this better
                }

                Err(e) => {
                    unreachable!("Error: {:?}", e);
                }
            }
        }

        rustc_middle::mir::Const::Val(val, ty) => match val {
            rustc_middle::mir::ConstValue::Scalar(scalar) => match scalar {
                rustc_const_eval::interpret::Scalar::Int(i) => {
                    return Expression::Constant { value: format!("{:?}", i).into() };
                }

                rustc_const_eval::interpret::Scalar::Ptr(_, _) => todo!("Ptr"),
            },

            _ => {
                warn!("Unhandled constant: {:?}", val);
                return Expression::NoOp {};
            }
        },

        _ => {
            warn!("Unhandled constant: {:?}", constant);
            return Expression::NoOp {};
        }
    }
}
