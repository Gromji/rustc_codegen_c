use std::fmt::{self, Debug};

use crate::{
    bb::{BasicBlock, BasicBlockIdentifier},
    crepr::Representable,
    definition::{CVarDecl, CVarDef},
    expression::{BinOpType, Expression},
    function::{CFunction, CodegenFunctionCx},
    stmt::Statement,
    structure::{self, CStructDef},
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
        panic!("Expected struct type as return of checked_op, return ty {:?}", place_ty);
    };

    if !fn_cx.ongoing_codegen.context.exists_header_fn_with_name(fn_name.as_str()) {
        debug!("Function for {fn_name} not found, creating one!");
        let ret_struct_name = format!(
            "{}_{}_{}",
            CFunction::RETURN_STRUCT_PREFIX,
            fn_name.as_str(),
            CFunction::RETURN_STRUCT_SUFFIX
        );

        let return_struct = structure::CComposite::Struct(structure::CStructDef {
            name: ret_struct_name,
            fields: vec![CVarDef::new(
                0,
                CFunction::RETURN_STRUCT_FIELD_NAME.to_string(),
                return_ty,
            )],
        });
        let ret_info = fn_cx.ongoing_codegen.context.add_composite(&return_struct);
        let return_ty = CType::Struct(ret_info);

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

        fn_cx.ongoing_codegen.context.get_mut_header_functions().push(checked_op);
    }
    span.exit();
    Expression::FnCall {
        function: Box::new(Expression::Constant { value: fn_name }),
        args: vec![lhs, rhs],
        is_builtin: false,
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
                if !fn_cx.ongoing_codegen.context.has_define_with_name(&max_uint) {
                    //#define UINT128_MAX (__uint128_t)(-1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(max_uint.clone(), "(__uint128_t)(-1)".to_string()));
                }
                if !fn_cx.ongoing_codegen.context.has_define_with_name(&max_int) {
                    //#define INT128_MAX (__int128_t)(UINT128_MAX >> 1)
                    fn_cx.ongoing_codegen.context.get_mut_defines().push(CDefine::new(
                        max_int.clone(),
                        format!("(__int128_t)({max_uint} >> 1)"),
                    ));
                    //#define INT128_MIN (__int128_t)(-INT128_MAX - 1)
                    fn_cx.ongoing_codegen.context.get_mut_defines().push(CDefine::new(
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
                if !fn_cx.ongoing_codegen.context.has_define_with_name(&max_uint) {
                    //#define UINT128_MAX (uint128_t)(-1)
                    fn_cx
                        .ongoing_codegen
                        .context
                        .get_mut_defines()
                        .push(CDefine::new(max_uint.clone(), "(uint128_t)(-1)".to_string()));
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
    return_ty: CType,
) -> CFunction {
    let field = c_struct.get_field(0);
    let sum_type = field.get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_ty);
    let (max_int_str, min_int_str) = extremum_val_of_type(fn_cx, sum_type);
    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "sum".to_string(), sum_type.clone()),
        Some(Expression::vari(0) + Expression::vari(1)),
    ));
    // (second > 0 && first > INT{bit_width}_MAX - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "overflow_a".to_string(), CType::Bool),
        Some(
            Expression::vari(1).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).gt(Expression::constant(&max_int_str) - Expression::vari(1)),
        ),
    ));
    // (second < 0 && first < INT{bit_width}_MIN - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow_b".to_string(), CType::Bool),
        Some(
            Expression::vari(1).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).lt(Expression::constant(&min_int_str) - Expression::vari(1)),
        ),
    ));
    // (second > 0 && first > INT{bit_width}_MAX - second) | (second < 0 && first < INT{bit_width}_MIN - second)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(3) | Expression::vari(4)),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));

    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(5)],
        ),
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

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "sum".to_string(), sum_type.clone()),
        Some(Expression::vari(0) + Expression::vari(1)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(2).lt(Expression::vari(0))),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    // sum < first
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(3)],
        ),
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
    let difference_type = field.get_type();
    let (max_int_str, min_int_str) = extremum_val_of_type(fn_cx, difference_type);
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), difference_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), difference_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "difference".to_string(), difference_type.clone()),
        Some(Expression::vari(0) - Expression::vari(1)),
    ));
    //second > 0 && first < INT{bit_width}_MIN + second
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "overflow_a".to_string(), CType::Bool),
        Some(
            Expression::vari(1).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).lt(Expression::constant(&min_int_str) + Expression::vari(1)),
        ),
    ));
    // second < 0 && first > INT{bit_width}_MAX + second
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow_b".to_string(), CType::Bool),
        Some(
            Expression::vari(1).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).gt(Expression::constant(&max_int_str) + Expression::vari(1)),
        ),
    ));
    //((second > 0 && first < INT{bit_width}_MIN + second) || (second < 0 && first > INT{bit_width}_MAX + second))
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow".to_string(), CType::Bool),
        Some(Expression::vari(3) | Expression::vari(4)),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(5)],
        ),
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

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), difference_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), difference_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "difference".to_string(), difference_type.clone()),
        Some(Expression::vari(0) - Expression::vari(1)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "underflow".to_string(), CType::Bool),
        Some(Expression::vari(0).lt(Expression::vari(1))),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    // first < second
    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(3)],
        ),
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

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), product_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), product_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "product".to_string(), product_type.clone()),
        Some(Expression::vari(0) * Expression::vari(1)),
    ));
    // (a == -1 && b == INT{}_MIN)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "overflow_a".to_string(), CType::Bool),
        Some(
            Expression::vari(0).equ(Expression::constant(&"-1".to_string()))
                & Expression::vari(1).equ(Expression::constant(&min_int_str)),
        ),
    ));
    // (b == -1 && a == INT{}_MIN)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(4, "overflow_b".to_string(), CType::Bool),
        Some(
            Expression::vari(1).equ(Expression::constant(&"-1".to_string()))
                & Expression::vari(0).equ(Expression::constant(&min_int_str)),
        ),
    ));
    // (a > 0 && b > 0 && a > INT{}_MAX / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(5, "overflow_c".to_string(), CType::Bool),
        Some(
            Expression::vari(0).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).gt(Expression::constant(&max_int_str) / Expression::vari(1)),
        ),
    ));
    // (a > 0 && b < 0 && b < INT{}_MIN / a)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(6, "overflow_d".to_string(), CType::Bool),
        Some(
            Expression::vari(0).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).lt(Expression::constant(&min_int_str) / Expression::vari(0)),
        ),
    ));
    // (a < 0 && b > 0 && a < INT{}_MIN / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(7, "overflow_e".to_string(), CType::Bool),
        Some(
            Expression::vari(0).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).gt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).lt(Expression::constant(&min_int_str) / Expression::vari(1)),
        ),
    ));
    // (a < 0 && b < 0 && a < INT{}_MAX / b)
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(8, "overflow_f".to_string(), CType::Bool),
        Some(
            Expression::vari(0).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(1).lt(Expression::constant(&"0".to_string()))
                & Expression::vari(0).lt(Expression::constant(&max_int_str) / Expression::vari(1)),
        ),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(9, "overflow".to_string(), CType::Bool),
        Some(
            Expression::vari(3)
                | Expression::vari(4)
                | Expression::vari(5)
                | Expression::vari(6)
                | Expression::vari(7)
                | Expression::vari(8),
        ),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));

    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(9)],
        ),
    }));

    c_fn.push_bb(bb);
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

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), product_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), product_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "product".to_string(), product_type.clone()),
        Some(Expression::vari(0) * Expression::vari(1)),
    ));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(3, "overflow".to_string(), CType::Bool),
        Some(
            Expression::vari(0).neq(Expression::constant(&"0".to_string()))
                & (Expression::vari(2) / (Expression::vari(0)).neq(Expression::vari(1))),
        ),
    ));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));

    bb.push(Statement::from_expression(Expression::Return {
        value: Expression::strct(
            Expression::constant(&c_struct.get_name()),
            vec![Expression::unbvari(2), Expression::unbvari(3)],
        ),
    }));
    c_fn.push_bb(bb);
    c_fn
}
