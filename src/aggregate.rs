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
    match *kind {
        AggregateKind::Tuple => match c_fn.get_local_decl(place.local.as_usize()).get_var_type() {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(tcx, ongoing_codegen, &field);
                    field_expressions.push(expression);
                }

                crepr::Expression::Struct {
                    name: struct_info.name.clone(),
                    fields: field_expressions,
                }
            }
            _ => {
                unreachable!("Error: Type should have been a struct!");
            }
        },
        _ => {
            warn!("Unhandled aggregate kind: {:?}", kind);
            crepr::Expression::NoOp {}
        }
    }
}
