use crate::crepr::{self, Expression};
use crate::definition::{CVarDecl, CVarDef};
use crate::function::CFunction;
use crate::stmt::handle_operand;
use crate::ty::{CStructInfo, CType};
use crate::{base::OngoingCodegen, crepr::indent};
use rustc_middle::mir::{AggregateKind, Operand, Place, Rvalue, StatementKind};
use tracing::{debug, debug_span, warn};
pub fn handle_aggregate<'tcx, I>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    c_fn: &CFunction,
    place: &Place<'tcx>,
    kind: &AggregateKind,
    fields: I,
) -> Expression
where
    I: 'tcx + Iterator<Item = &'tcx Operand<'tcx>>,
{
    let var_idx = place.local.as_usize();
    let local_var = c_fn.get_local_var(var_idx);
    match *kind {
        AggregateKind::Tuple => match local_var.get_var_type() {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(tcx, ongoing_codegen, &field);
                    field_expressions.push(expression);
                }

                let rhs = crepr::Expression::Struct {
                    name: struct_info.name.clone(),
                    fields: field_expressions,
                };
                let lhs = crepr::Expression::Variable { local: var_idx, idx: None };
                crepr::Expression::Assignment { lhs: Box::new(lhs), rhs: Box::new(rhs) }
            }
            _ => {
                unreachable!("Error: Type should have been a struct!");
            }
        },
        AggregateKind::Array(ty) => {
            let var_name = local_var.get_name();
            let mut field_expressions = Vec::new();
            for (i, field) in fields.enumerate() {
                let rh_expression = handle_operand(tcx, ongoing_codegen, &field);
                let lh_expression = crepr::Expression::Variable { local: var_idx, idx: Some(i) };
                field_expressions.push(crepr::Expression::Assignment {
                    lhs: Box::new(lh_expression),
                    rhs: Box::new(rh_expression),
                });
            }

            crepr::Expression::Array { fields: field_expressions }
        }
        _ => {
            warn!("Unhandled aggregate kind: {:?}", kind);
            crepr::Expression::NoOp {}
        }
    }
}
