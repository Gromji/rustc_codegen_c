use crate::expression::Expression;
use crate::function::{CFunction, CodegenFunctionCx};
use crate::stmt::{handle_operand, handle_place};
use crate::ty::CType;
use rustc_middle::mir::{AggregateKind, Operand, Place};
use tracing::{debug_span, warn};
pub fn handle_aggregate<'tcx, 'ccx, I>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    place: &Place<'tcx>,
    kind: &AggregateKind<'tcx>,
    fields: I,
) -> Expression
where
    I: 'tcx + Iterator<Item = &'tcx Operand<'tcx>>,
{
    let span = debug_span!("handle_aggregate").entered();
    let var_idx = place.local.as_usize();
    let local_var = c_fn.get_local_var(var_idx);
    let var_type = local_var.get_type();
    let result = match *kind {
        AggregateKind::Tuple => match var_type {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }

                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant { value: struct_info.name.clone() }),
                    fields: field_expressions,
                };
                
                let lhs = handle_place(fn_cx, place);
                Expression::Assignment { lhs: Box::new(lhs), rhs: Box::new(rhs) }
            }
            _ => {
                unreachable!("Error: Type should have been a struct!");
            }
        },
        AggregateKind::Array(_) => {
            let mut field_expressions = Vec::new();
            for (i, field) in fields.enumerate() {
                let rh_expression = handle_operand(fn_cx, &field);
                let lh_expression = Expression::arr_vari(var_idx, i); // Luka: might not be appropriate to directly create an array variable here, also unsure if i is right
                field_expressions.push(Expression::Assignment {
                    lhs: lh_expression,
                    rhs: Box::new(rh_expression),
                });
            }

            Expression::Array { fields: field_expressions }
        }
        AggregateKind::Adt(_, _, _, _, _) => match var_type {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }
                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant { value: struct_info.name.clone() }),
                    fields: field_expressions,
                };
                let lhs = handle_place(fn_cx, place);
                Expression::Assignment { lhs: Box::new(lhs), rhs: Box::new(rhs) }
            }
            _ => {
                warn!("Unhandled aggregate kind: {:?}", kind);
                Expression::NoOp {}
            }
        },
        _ => {
            warn!("Unhandled aggregate kind: {:?}", kind);
            Expression::NoOp {}
        }
    };
    span.exit();
    result
}
