use crate::function::CFunction;
use crate::{base::OngoingCodegen, write};
use rustc_middle::mir::{Place, Rvalue, StatementKind};
use std::io::Write;

pub fn handle_stmt<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    stmt: &rustc_middle::mir::Statement<'tcx>,
    c_fn: &mut CFunction,
) {
    writeln!(std::io::stdout(), "Statement: {:?}", stmt).unwrap();
    writeln!(std::io::stdout(), "Statement Kind: {:?}", stmt.kind).unwrap();
    match &stmt.kind {
        StatementKind::Assign(val) => {
            handle_assign(tcx, ongoing_codegen, &val.0, &val.1, c_fn);
        }
        _ => {}
    }
}

// TODO: Decompose this function
fn handle_assign<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    place: &Place<'tcx>,
    rvalue: &Rvalue<'tcx>,
    c_fn: &mut CFunction,
) {
    writeln!(std::io::stdout(), "Place: {:?}", place).unwrap();
    writeln!(std::io::stdout(), "Rvalue: {:?}", rvalue).unwrap();

    let mut c_assign = String::new();
    c_assign.push_str(format!("var{} = ", place.local.as_usize()).as_str());

    match rvalue {
        Rvalue::Use(operand) => {
            // Match the operand
            match operand {
                rustc_middle::mir::Operand::Copy(place) => {
                    c_assign.push_str(format!("var{};", place.local.as_usize()).as_str());
                }
                rustc_middle::mir::Operand::Move(place) => {
                    todo!("Move");
                }
                rustc_middle::mir::Operand::Constant(constant) => match constant.const_ {
                    rustc_middle::mir::Const::Unevaluated(c, t) => {
                        match tcx.const_eval_poly(c.def) {
                            Ok(val) => {
                                c_assign.push_str(format!("{:?};", val).as_str());
                            }
                            Err(e) => {
                                panic!("Error: {:?}", e);
                            }
                        }
                    }
                    rustc_middle::mir::Const::Val(val, ty) => match val {
                        rustc_middle::mir::ConstValue::Scalar(scalar) => match scalar {
                            rustc_const_eval::interpret::Scalar::Int(i) => {
                                c_assign.push_str(format!("{:?};", i).as_str())
                            }
                            rustc_const_eval::interpret::Scalar::Ptr(_, _) => todo!("Ptr"),
                        },
                        _ => {}
                    },
                    _ => {}
                },
            }
        }
        _ => {}
    }

    c_fn.push(c_assign.as_str(), true, 1);
}
