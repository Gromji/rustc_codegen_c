use core::panic;

use crate::expression::Expression;
use crate::function::{CFunction, CodegenFunctionCx};
use crate::stmt::{handle_operand, handle_place};
use crate::ty::CType;
use crate::utils::truncate_to_size;
use rustc_index::IndexVec;
use rustc_middle::mir::{AggregateKind, Operand, Place};
use rustc_target::abi::FieldIdx;
use tracing::{debug, debug_span, error, warn};
pub fn handle_aggregate<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    place: &Place<'tcx>,
    kind: &AggregateKind<'tcx>,
    fields: &IndexVec<FieldIdx, Operand<'tcx>>,
) -> Expression {
    let span = debug_span!("handle_aggregate").entered();
    let var_idx = place.local.as_usize();

    let ty = fn_cx.ty_for_local(place.local);
    let local_var: &crate::definition::CVarDef = c_fn.get_local_var(var_idx);
    let var_type = local_var.get_type();
    let result = match *kind {
        AggregateKind::Tuple => match var_type {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }

                debug!("Tuple with fields {:?}", field_expressions);
                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant {
                        value: struct_info.name.clone(),
                    }),
                    fields: field_expressions,
                };

                let lhs = handle_place(fn_cx, place);
                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            _ => {
                debug!("Non struct encountered in aggregate, must be unboxed tuple");
                assert!(fields.len() == 1);
                let expression = handle_operand(fn_cx, fields.iter().next().unwrap());
                let lhs = handle_place(fn_cx, place);
                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(expression),
                }
            }
        },
        AggregateKind::Array(_) => {
            let mut field_expressions = Vec::new();
            for (i, field) in fields.iter().enumerate() {
                let rh_expression = handle_operand(fn_cx, &field);
                let lh_expression = Expression::arr_vari(var_idx, i); // Luka: might not be appropriate to directly create an array variable here, also unsure if i is right
                field_expressions.push(Expression::Assignment {
                    lhs: lh_expression,
                    rhs: Box::new(rh_expression),
                });
            }

            Expression::Array {
                fields: field_expressions,
            }
        }

        AggregateKind::Adt(_, variant_idx, _, _, active_field_idx) => match var_type {
            CType::Struct(struct_info) => {
                let mut field_expressions = Vec::new();
                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }
                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant {
                        value: struct_info.name.clone(),
                    }),
                    fields: field_expressions,
                };
                let lhs = handle_place(fn_cx, place);
                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }

            CType::Union(union_info) => {
                let union_def = fn_cx
                    .ongoing_codegen
                    .context
                    .get_composite(union_info)
                    .as_struct_def();

                let active_field = &union_def.fields[active_field_idx.unwrap().as_usize()];

                let field_expressions = match fields.len() {
                    0 => {
                        vec![]
                    }

                    1 => {
                        vec![(active_field.get_name(), handle_operand(fn_cx, fields.iter().next().unwrap()))]
                    }

                    _ => {
                        panic!("Union with more than one field. I will kms if this happens.")
                    }
                };

                let rhs = Expression::NamedStruct {
                    name: Box::new(Expression::Constant {
                        value: union_def.name.clone(),
                    }),

                    fields: field_expressions
                };

                let lhs = handle_place(fn_cx, place);
                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }

            CType::TaggedUnion(union_info) => {
                let mut field_expressions = Vec::new();

                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }

                let discriminant = ty.discriminant_for_variant(fn_cx.tcx, variant_idx).unwrap();

                let t_union_def = fn_cx
                    .ongoing_codegen
                    .context
                    .get_composite(union_info)
                    .as_tagged_union_def();

                let union_info = t_union_def.union_var.get_type().as_composite_info();
                let union_def = fn_cx
                    .ongoing_codegen
                    .context
                    .get_composite(&union_info)
                    .as_struct_def();

                let union_variant_field = &union_def.fields[variant_idx.as_usize()];
                let union_variant_struct = fn_cx
                    .ongoing_codegen
                    .context
                    .get_composite(&union_variant_field.get_type().as_composite_info())
                    .as_struct_def();

                let outside_field_expressions = vec![
                    // Tag field (discriminant)
                    Expression::Constant {
                        value: format!(
                            "{}",
                            truncate_to_size(
                                discriminant.val,
                                discriminant.ty.int_size_and_signed(fn_cx.tcx).0.bytes() as usize
                            )
                        ),
                    },
                    Expression::NamedStruct {
                        name: Box::new(Expression::Constant {
                            value: union_info.name.clone(),
                        }),

                        fields: vec![(
                            union_variant_field.get_name(),
                            Expression::Struct {
                                name: Box::new(Expression::Constant {
                                    value: union_variant_struct.name.clone(),
                                }),
                                fields: field_expressions,
                            },
                        )],
                    },
                ];

                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant {
                        value: t_union_def.name.clone(),
                    }),
                    fields: outside_field_expressions,
                };

                let lhs = handle_place(fn_cx, place);

                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }

            _ => {
                error!("Unhandled aggregate kind: {:?}", kind);
                Expression::NoOp {}
            }
        },

        AggregateKind::Closure(def, args) => {
            let closure = args.as_closure();
            debug!(
                "Closure encountered, with defid: {:?}, args: {:?}",
                def, closure
            );

            let closure_arg_struct = fn_cx.rust_to_c_type(&closure.tupled_upvars_ty());

            if let CType::Struct(struct_info) = closure_arg_struct {
                let mut field_expressions = Vec::new();

                for field in fields {
                    let expression = handle_operand(fn_cx, &field);
                    field_expressions.push(expression);
                }

                let rhs = Expression::Struct {
                    name: Box::new(Expression::Constant {
                        value: struct_info.name.clone(),
                    }),
                    fields: field_expressions,
                };

                let lhs = handle_place(fn_cx, place);
                Expression::Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            } else {
                panic!("Expected closure arg struct to be a struct");
            }
        }

        _ => {
            warn!("Unhandled aggregate kind: {:?}", kind);
            Expression::NoOp {}
        }
    };
    span.exit();
    result
}
