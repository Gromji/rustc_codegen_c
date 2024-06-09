use crate::aggregate::handle_aggregate;
use crate::crepr::{indent, Representable, RepresentationContext};
use crate::expression::{Expression, VariableAccess};
use crate::function::{CFunction, CodegenFunctionCx};
use crate::header::handle_checked_op;
use crate::structure::CTaggedUnionDef;
use crate::ty::CType;
use crate::utils;
use rustc_middle::mir::{
    BinOp, CastKind, ConstOperand, ConstValue, Operand, Place, Rvalue, StatementKind,
};
use rustc_middle::ty::{ParamEnv, Ty};
use std::fmt::{self, Debug};
use tracing::{debug, debug_span, error, span, warn};

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
    fn repr(&self, f: &mut (dyn fmt::Write), context: &mut RepresentationContext) -> fmt::Result {
        if context.include_comments {
            if let Some(comment) = &self.comment {
                indent(f, context)?;
                write!(f, "/* {} */\n", comment)?;
            }
        }

        if let Some(expression) = &self.expression {
            indent(f, context)?;
            expression.repr(f, context)?;

            if !matches!(expression, Expression::NoOp {}) {
                write!(f, ";")?;
            }

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
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    stmt: &'ccx rustc_middle::mir::Statement<'tcx>,
) -> Statement {
    let span = debug_span!("handle_stmt").entered();

    debug!("Statement: {:?}", stmt);
    debug!("Kind: {:?}", stmt.kind);

    let expression = match &stmt.kind {
        StatementKind::Assign(val) => {
            let exp = handle_assign(fn_cx, c_fn, &val.0, &val.1);

            if let Expression::Assignment { lhs: _, rhs } = &exp {
                if matches!(**rhs, Expression::NoOp {}) { Expression::NoOp {} } else { exp }
            } else {
                exp
            }
        }

        _ => Expression::NoOp {},
    };

    let statement = Statement::new(expression, format!("//{:?}", stmt).into());

    span.exit();

    return statement;
}

pub fn handle_place<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    place: &Place<'tcx>,
) -> Expression {
    // if the projection is empty, we can just return the variable
    let _span = span!(tracing::Level::DEBUG, "handle_place").entered();
    debug!("place: {:?}, projections {:?}", place, place.projection);

    let mut access = Vec::new();

    let current_ty = fn_cx.ty_for_local(place.local);

    let mut ctype =
        fn_cx.ctype_from_cache(&current_ty).expect("No ctype found in cache for rust type");

    for proj in place.projection {
        match proj {
            rustc_middle::mir::ProjectionElem::Field(field, ty) => {
                debug!(
                    "Field: {:?}, Type: {:?}, current_ty: {:?}, next_ty: {:?}",
                    field, ctype, current_ty, ty
                );

                match ctype {
                    CType::Struct(info) | CType::Union(info) => {
                        access.push(VariableAccess::Field {
                            name: fn_cx
                                .ongoing_codegen
                                .context
                                .get_field_name_for_struct(&info, field.as_usize())
                                .unwrap(),
                        });

                        // shortcut to get the ctype of the next field
                        ctype = fn_cx
                            .ctype_from_cache(&ty)
                            .expect("No ctype found in cache for rust type");
                    }
                    _ => {
                        error!("Expected struct type, got {:?}", ctype);
                    }
                }
            }
            rustc_middle::mir::ProjectionElem::Index(idx_local) => {
                access.push(VariableAccess::Index {
                    expression: Expression::unbvari(idx_local.as_usize()),
                })
            }

            rustc_middle::mir::ProjectionElem::ConstantIndex { offset, .. } => {
                access.push(VariableAccess::Index {
                    expression: Expression::const_int(offset as i128),
                });
            }

            rustc_middle::mir::ProjectionElem::Subslice { .. } => {
                todo!("Subslice")
            }

            rustc_middle::mir::ProjectionElem::Downcast(_, variant_idx) => {
                match ctype {
                    CType::TaggedUnion(union_info) => {
                        // access the union field
                        access.push(VariableAccess::Field {
                            name: CTaggedUnionDef::UNION_NAME.to_string(),
                        });

                        // find the struct of the actual union
                        let t_union_def = fn_cx
                            .ongoing_codegen
                            .context
                            .get_composite(&union_info)
                            .as_tagged_union_def();
                        let union_def = fn_cx
                            .ongoing_codegen
                            .context
                            .get_composite(&t_union_def.union_var.get_type().as_composite_info())
                            .as_struct_def();

                        // access the variant
                        let variant_field = &union_def.fields[variant_idx.as_usize()];
                        access.push(VariableAccess::Field { name: variant_field.get_name() });

                        // set ctype to correct variant
                        ctype = variant_field.get_type().clone();
                    }

                    _ => {
                        // unsure what other types might get downcast, for now just panic
                        panic!("Expected tagged union type, got {:?}", ctype);
                    }
                }
            }

            rustc_middle::mir::ProjectionElem::Deref => {
                access.push(VariableAccess::Dereference);

                // Another shortcut
                ctype = fn_cx
                    .ctype_from_cache(&current_ty.builtin_deref(true).unwrap())
                    .expect("No ctype found in cache for rust type");
            }

            rustc_middle::mir::ProjectionElem::OpaqueCast(_) => {
                error!("OpaqueCast")
            }

            _ => {}
        }
    }

    return Expression::Variable { local: place.local.as_usize(), access };
}

pub fn handle_operand<'tcx, 'ccx>(
    fn_cx: &CodegenFunctionCx<'tcx, 'ccx>,
    operand: &Operand<'tcx>,
) -> Expression {
    match operand {
        Operand::Copy(place) => handle_place(fn_cx, place),
        // move operations can be treated as a copy operation (I think)
        Operand::Move(place) => handle_place(fn_cx, place),
        Operand::Constant(constant) => handle_constant(fn_cx, constant),
    }
}

fn handle_assign<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    c_fn: &CFunction,
    place: &Place<'tcx>,
    rvalue: &'ccx Rvalue<'tcx>,
) -> Expression {
    let span = debug_span!("handle_assign").entered();
    debug!("place( {:?} )", place);
    debug!("rvalue( {:?} )", rvalue);

    let expression = match rvalue {
        Rvalue::Use(operand) => {
            debug!("Assign USE: {:?}", operand);
            handle_operand(fn_cx, operand)
        }

        Rvalue::BinaryOp(op, operands) => {
            debug!("Assign BINARY OP: {:?}", op);
            let lhs = handle_operand(fn_cx, &operands.0);
            let rhs = handle_operand(fn_cx, &operands.1);
            let ty = operands.0.ty(&fn_cx.mir.local_decls, fn_cx.tcx);

            let place_ty = fn_cx.ty_for_local(place.local);

            match op {
                BinOp::AddWithOverflow | BinOp::SubWithOverflow | BinOp::MulWithOverflow => {
                    handle_checked_op(fn_cx, op.into(), lhs, rhs, &ty, &place_ty)
                }
                _ => Expression::BinaryOp { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) },
            }
        }
        Rvalue::Aggregate(kind, fields) => {
            debug!("Assign AGGREGATE: {:?}", kind);
            // Return instantly because it already handles assignments.
            return handle_aggregate(fn_cx, c_fn, place, kind, fields);
        }

        Rvalue::Ref(_region, _kind, place) => {
            debug!("Assign REF: {:?}", place);
            let place = handle_place(fn_cx, place);
            if let Expression::Variable { local, access } = place {
                let mut new_access = access.clone();
                new_access.push(VariableAccess::Reference);
                Expression::Variable { local, access: new_access }
            } else {
                panic!("Expected place to be a variable");
            }
        }

        Rvalue::CopyForDeref(place) => {
            debug!("Assign COPY FOR DEREF: {:?}", place);

            handle_place(fn_cx, place)
        }

        Rvalue::Discriminant(place) => {
            debug!("Assign DISCRIMINANT: {:?}", place);

            if let Expression::Variable { local, access } = handle_place(fn_cx, place) {
                let mut modified_access = access;
                modified_access
                    .push(VariableAccess::Field { name: CTaggedUnionDef::TAG_NAME.to_string() });
                Expression::Variable { local, access: modified_access }
            } else {
                unreachable!("non variable for handle_place")
            }
        }

        Rvalue::Repeat(operand, len) => {
            debug!("Assign REPEAT: {:?} {:?}. Ignoring", operand, len);
            Expression::NoOp {}
        }

        Rvalue::Cast(kind, operand, ty) => {
            debug!("Assign CAST: {:?} {:?} {:?}", kind, operand, ty);
            handle_cast(fn_cx, kind, operand, ty)
        }

        _ => {
            warn!("Unhandled rvalue: {:?}", rvalue);
            Expression::NoOp {}
        }
    };

    span.exit();

    return Expression::Assignment {
        lhs: Box::new(handle_place(fn_cx, place)),
        rhs: Box::new(expression),
    };
}

pub fn handle_cast<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    kind: &CastKind,
    operand: &Operand<'tcx>,
    ty: &Ty<'tcx>,
) -> Expression {
    debug!("Cast type: {:?}", kind);
    debug!("Operand: {:?}", operand);
    debug!("Type: {:?}", ty);

    let operand = handle_operand(fn_cx, operand);

    match kind {
        CastKind::PointerExposeProvenance => todo!("Unhandled cast: {:?}", kind),
        CastKind::PointerWithExposedProvenance => todo!("Unhandled cast: {:?}", kind),
        CastKind::PointerCoercion(_) => todo!("Unhandled cast: {:?}", kind),
        CastKind::DynStar => todo!("Unhandled cast: {:?}", kind),
        CastKind::IntToInt
        | CastKind::FloatToInt
        | CastKind::FloatToFloat
        | CastKind::IntToFloat => {
            Expression::Cast { ty: fn_cx.rust_to_c_type(ty), value: Box::new(operand) }
        }
        CastKind::PtrToPtr => todo!("Unhandled cast: {:?}", kind),
        CastKind::FnPtrToPtr => todo!("Unhandled cast: {:?}", kind),
        CastKind::Transmute => todo!("Unhandled cast: {:?}", kind),
    }
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
    let _span = span!(tracing::Level::DEBUG, "handle_const_value").entered();
    debug!("Const value: {:?}, with type: {:?}", val, ty);

    match val {
        rustc_middle::mir::ConstValue::Scalar(scalar) => match scalar {
            rustc_const_eval::interpret::Scalar::Int(_i) => match ty.kind() {
                rustc_middle::ty::TyKind::Int(_) => {
                    return Expression::Constant {
                        value: format!("{}", utils::scalar_to_u128(&scalar)),
                    };
                }
                rustc_middle::ty::TyKind::Uint(_) => {
                    return Expression::Constant {
                        value: format!("{}u", utils::scalar_to_u128(&scalar)),
                    };
                }
                rustc_middle::ty::TyKind::Float(_) => {
                    return Expression::Constant {
                        value: format!("{}", utils::scalar_to_float(&scalar)),
                    };
                }

                rustc_middle::ty::TyKind::Bool => {
                    return Expression::Constant {
                        value: format!("{}", scalar.to_bool().unwrap()),
                    };
                }

                _ => {
                    warn!("Unhandled scalar kind: {:?}", ty.kind());
                    return Expression::NoOp {};
                }
            },

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
