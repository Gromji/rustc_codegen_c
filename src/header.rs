use std::fmt::{self, Debug};

use crate::{
    bb::{BasicBlock, BasicBlockIdentifier},
    crepr::Representable,
    definition::{CVarDecl, CVarDef},
    expression::{BinOpType, Expression},
    function::{CFunction, CodegenFunctionCx},
    stmt::Statement,
    structure::CStructDef,
    ty::CType,
};
use rustc_middle::ty::Ty;
use tracing::{debug, debug_span};

// We will need to change structure of CDefine for more versitile use
pub struct CDefine {
    name: String,
    value: String,
}
impl Representable for CDefine {
    fn repr(
        &self,
        f: &mut (dyn fmt::Write),
        _context: &mut crate::crepr::RepresentationContext,
    ) -> fmt::Result {
        write!(f, "#define {} {}", self.name, self.value)
    }
}

impl Debug for CDefine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}
impl CDefine {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub fn handle_checked_op<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    op: BinOpType,
    lhs: Expression,
    rhs: Expression,
    ty: &Ty<'tcx>,
    place_ty: &Ty<'tcx>,
) -> Expression {
    let span = debug_span!("handle_checked_op").entered();
    // We can change the naming of the functions later
    let fn_name = format!("{op}_{ty:?}");

    let return_ty = fn_cx.rust_to_c_type(place_ty);
    let c_struct: CStructDef = if let CType::Struct(info) = return_ty.clone() {
        fn_cx
            .ongoing_codegen
            .context
            .get_struct_def(&info)
            .expect("Struct not found for provided info")
    } else {
        panic!(
            "Expected struct type as return of checked_op, return ty {:?}",
            place_ty
        );
    };

    if !fn_cx
        .ongoing_codegen
        .context
        .exists_header_fn_with_name(fn_name.as_str())
    {
        debug!("Function for {fn_name} not found, creating one!");
        let checked_op = match op {
            BinOpType::CheckedAdd => {
                if ty.is_signed() {
                    signed_add(fn_cx, &fn_name, &c_struct, return_ty)
                } else {
                    unsigned_add(fn_cx, &fn_name, &c_struct, return_ty)
                }
            }
            BinOpType::CheckedSub => {
                if ty.is_signed() {
                    signed_sub(fn_cx, &fn_name, &c_struct, return_ty)
                } else {
                    unsigned_sub(fn_cx, &fn_name, &c_struct, return_ty)
                }
            }
            BinOpType::CheckedMul => {
                if ty.is_signed() {
                    signed_mul(fn_cx, &fn_name, &c_struct, return_ty)
                } else {
                    unsigned_mul(fn_cx, &fn_name, &c_struct, return_ty)
                }
            }
            _ => {
                todo!("Checked operation not handled: {:?}", op);
            }
        };
        fn_cx
            .ongoing_codegen
            .context
            .get_mut_header_functions()
            .push(checked_op);
    }
    span.exit();
    Expression::FnCall {
        function: Box::new(Expression::Constant { value: fn_name }),
        args: vec![lhs, rhs],
    }
}

fn extremum_val_of_type<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    ty: &CType,
) -> (String, String) {
    match ty {
        CType::Int(c_int_ty) => {
            let bit_width = c_int_ty.bit_width();
            let max_uint = format!("UINT{bit_width}_MAX");
            let max_int = format!("INT{bit_width}_MAX");
            let min_int = format!("INT{bit_width}_MIN");
            if bit_width == 128 {
                if !fn_cx
                    .ongoing_codegen
                    .context
                    .has_define_with_name(&max_uint)
                {
                    //#define UINT128_MAX (__uint128_t)(-1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(
                            max_uint.clone(),
                            "(__uint128_t)(-1)".to_string(),
                        ));
                }
                if !fn_cx.ongoing_codegen.context.has_define_with_name(&max_int) {
                    //#define INT128_MAX (__int128_t)(UINT128_MAX >> 1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(
                            max_int.clone(),
                            format!("(__int128_t)({max_uint} >> 1)"),
                        ));
                    //#define INT128_MIN (__int128_t)(-INT128_MAX - 1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(
                            min_int.clone(),
                            "(__int128_t)(-INT128_MAX - 1)".to_string(),
                        ));
                }
            }
            (max_int, min_int)
        }
        CType::UInt(c_uint_ty) => {
            let bit_width = c_uint_ty.bit_width();
            let max_uint = format!("UINT{bit_width}_MAX");
            if bit_width == 128 {
                if !fn_cx
                    .ongoing_codegen
                    .context
                    .has_define_with_name(&max_uint)
                {
                    //#define UINT128_MAX (uint128_t)(-1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(
                            max_uint.clone(),
                            "(uint128_t)(-1)".to_string(),
                        ));
                }
            }
            (max_uint, "0".to_string())
        }
        _ => panic!("Expected integer type, found {:?}", ty),
    }
}

fn signed_add<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let sum_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    let (max_int_str, min_int_str) = extremum_val_of_type(fn_cx, sum_type);
    c_fn.add_signature_var(CVarDef::new(1, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(2, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "sum".to_string(), sum_type.clone()),
        Some(Expression::vari(1) + Expression::vari(2)),
    ));
    // (second > 0 && first > INT{bit_width}_MAX - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow_a".to_string(), CType::Bool),
        Some(
            Expression::vari(2).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).gt(Expression::constant(&max_int_str) - Expression::vari(2)),
        ),
    ));
    // (second < 0 && first < INT{bit_width}_MIN - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow_b".to_string(), CType::Bool),
        Some(
            Expression::vari(2).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).lt(Expression::constant(&min_int_str) - Expression::vari(2)),
        ),
    ));
    // (second > 0 && first > INT{bit_width}_MAX - second) | (second < 0 && first < INT{bit_width}_MIN - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(6, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(4) | Expression::vari(5)),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(6)],
        ),
    )));

    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn unsigned_add<'tcx, 'ccx>(
    _fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let sum_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());

    c_fn.add_signature_var(CVarDef::new(1, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(2, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "sum".to_string(), sum_type.clone()),
        Some(Expression::vari(1) + Expression::vari(2)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(3).lt(Expression::vari(1))),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(4)],
        ),
    )));
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn signed_sub<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let diff_type = field.get_type();
    let (max_int_str, min_int_str) = extremum_val_of_type(fn_cx, diff_type);
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    c_fn.add_signature_var(CVarDef::new(1, "first".to_string(), diff_type.clone()));
    c_fn.add_signature_var(CVarDef::new(2, "second".to_string(), diff_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "difference".to_string(), diff_type.clone()),
        Some(Expression::vari(1) - Expression::vari(2)),
    ));
    //second > 0 && first < INT{bit_width}_MIN + second
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow_a".to_string(), CType::Bool),
        Some(
            Expression::vari(2).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).lt(Expression::constant(&min_int_str) + Expression::vari(2)),
        ),
    ));
    // second < 0 && first > INT{bit_width}_MAX + second
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow_b".to_string(), CType::Bool),
        Some(
            Expression::vari(2).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).gt(Expression::constant(&max_int_str) + Expression::vari(2)),
        ),
    ));
    //((second > 0 && first < INT{bit_width}_MIN + second) || (second < 0 && first > INT{bit_width}_MAX + second))
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(6, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(4) | Expression::vari(5)),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(6)],
        ),
    )));
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn unsigned_sub<'tcx, 'ccx>(
    _fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let difference_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());

    c_fn.add_signature_var(CVarDef::new(
        1,
        "first".to_string(),
        difference_type.clone(),
    ));
    c_fn.add_signature_var(CVarDef::new(
        2,
        "second".to_string(),
        difference_type.clone(),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "difference".to_string(), difference_type.clone()),
        Some(Expression::vari(1) - Expression::vari(2)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "underflow".to_string(), CType::Bool),
        Some(Expression::vari(1).lt(Expression::vari(2))),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(1));
    bb.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(4)],
        ),
    )));
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn signed_mul<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let product_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    let (max_int_str, min_int_str) = extremum_val_of_type(fn_cx, product_type);

    c_fn.add_signature_var(CVarDef::new(1, "first".to_string(), product_type.clone()));
    c_fn.add_signature_var(CVarDef::new(2, "second".to_string(), product_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "product".to_string(), product_type.clone()),
        Some(Expression::vari(1) * Expression::vari(2)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "is_zero".to_string(), CType::Bool),
        Some(
            Expression::vari(1).equ(Expression::constant(&"0".to_string()))
                | Expression::vari(2).equ(Expression::constant(&"0".to_string())),
        ),
    ));

    // (a == -1 && b == INT{}_MIN)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow_a".to_string(), CType::Bool),
        None,
    ));
    // (b == -1 && a == INT{}_MIN)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(6, "overflow_b".to_string(), CType::Bool),
        None,
    ));
    // (a > 0 && b > 0 && a > INT{}_MAX / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(7, "overflow_c".to_string(), CType::Bool),
        None,
    ));
    // (a > 0 && b < 0 && b < INT{}_MIN / a)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(8, "overflow_d".to_string(), CType::Bool),
        None,
    ));
    // (a < 0 && b > 0 && a < INT{}_MIN / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(9, "overflow_e".to_string(), CType::Bool),
        None,
    ));
    // (a < 0 && b < 0 && a < INT{}_MAX / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(10, "overflow_f".to_string(), CType::Bool),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(11, "overflow".to_string(), CType::Bool),
        Some(Box::new(Expression::const_int(0))),
    ));
    let mut bb0 = BasicBlock::new(BasicBlockIdentifier(0));
    bb0.push(Statement::from_expression(Expression::SwitchJump {
        value: Expression::vari(4),
        cases: vec![(Box::new(Expression::const_int(0)), BasicBlockIdentifier(1))],
        default: BasicBlockIdentifier(2),
    }));

    let mut bb1 = BasicBlock::new(BasicBlockIdentifier(1));
    bb1.push(Statement::from_expression(Expression::unbvari(5).assign(
        Expression::vari(1).equ(Expression::constant(&"-1".to_string()))
            & Expression::vari(2).equ(Expression::constant(&min_int_str)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(6).assign(
        Expression::vari(2).equ(Expression::constant(&"-1".to_string()))
            & Expression::vari(1).equ(Expression::constant(&min_int_str)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(7).assign(
        Expression::vari(1).gt(Expression::constant(&"0".to_string()))
            & Expression::vari(2).gt(Expression::constant(&"0".to_string()))
            & Expression::vari(1).gt(Expression::constant(&max_int_str) / Expression::vari(2)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(8).assign(
        Expression::vari(1).gt(Expression::constant(&"0".to_string()))
            & Expression::vari(2).lt(Expression::constant(&"0".to_string()))
            & Expression::vari(3).lt(Expression::constant(&min_int_str) / Expression::vari(1)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(9).assign(
        Expression::vari(1).lt(Expression::constant(&"0".to_string()))
            & Expression::vari(2).gt(Expression::constant(&"0".to_string()))
            & Expression::vari(1).lt(Expression::constant(&min_int_str) / Expression::vari(2)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(10).assign(
        Expression::vari(1).lt(Expression::constant(&"0".to_string()))
            & Expression::vari(2).lt(Expression::constant(&"0".to_string()))
            & Expression::vari(1).lt(Expression::constant(&max_int_str) / Expression::vari(2)),
    )));
    bb1.push(Statement::from_expression(Expression::unbvari(11).assign(
        Expression::vari(5)
            | Expression::vari(6)
            | Expression::vari(7)
            | Expression::vari(8)
            | Expression::vari(9)
            | Expression::vari(10),
    )));

    let mut bb2 = BasicBlock::new(BasicBlockIdentifier(2));
    bb2.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(11)],
        ),
    )));
    bb2.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));

    c_fn.push_bb(bb0);
    c_fn.push_bb(bb1);
    c_fn.push_bb(bb2);
    c_fn
}

fn unsigned_mul<'tcx, 'ccx>(
    _fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    fn_name: &String,
    c_struct: &CStructDef,
    return_type: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let product_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());

    c_fn.add_signature_var(CVarDef::new(1, "first".to_string(), product_type.clone()));
    c_fn.add_signature_var(CVarDef::new(2, "second".to_string(), product_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(0, "var0".to_string(), return_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "product".to_string(), product_type.clone()),
        Some(Expression::vari(1) * Expression::vari(2)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "is_zero".to_string(), CType::Bool),
        Some(Expression::vari(1).equ(Expression::constant(&"0".to_string()))),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow".to_string(), CType::Bool),
        Some(Expression::constant(&"false".to_string())),
    ));
    let mut bb0 = BasicBlock::new(BasicBlockIdentifier(0));
    bb0.push(Statement::from_expression(Expression::SwitchJump {
        value: Expression::vari(4),
        cases: vec![(Box::new(Expression::const_int(0)), BasicBlockIdentifier(1))],
        default: BasicBlockIdentifier(2),
    }));
    let mut bb1 = BasicBlock::new(BasicBlockIdentifier(1));
    bb1.push(Statement::from_expression(Expression::unbvari(5).assign(
        Expression::vari(3) / (Expression::vari(1)).neq(Expression::vari(2)),
    )));

    let mut bb2 = BasicBlock::new(BasicBlockIdentifier(2));
    bb2.push(Statement::from_expression(Expression::assign(
        &Expression::unbvari(0),
        Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(3), Expression::unbvari(5)],
        ),
    )));
    bb2.push(Statement::from_expression(Expression::Return {
        value: Expression::vari(0),
    }));
    c_fn.push_bb(bb0);
    c_fn.push_bb(bb1);
    c_fn.push_bb(bb2);
    c_fn
}
