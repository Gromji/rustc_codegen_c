use crate::{
    bb::{BasicBlock, BasicBlockIdentifier},
    definition::{CVarDecl, CVarDef},
    expression::{BinOpType, Expression},
    function::{CFunction, CodegenFunctionCx},
    stmt::Statement,
    structure::CStruct,
    ty::{rust_to_c_type, CStructInfo, CType},
};
use rustc_middle::ty::Ty;
use tracing::{debug_span, warn};

macro_rules! strct {
    ($lhs:expr, $rhs:expr) => {
        Box::new(Expression::Struct { name: $lhs, fields: $rhs })
    };
}

macro_rules! assignment {
    ($left:expr, $right:expr) => {
        Expression::Assignment { lhs: $left, rhs: $right }
    };
}
macro_rules! add {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::Add, lhs: $left, rhs: $right })
    };
}
macro_rules! sub {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::Sub, lhs: $left, rhs: $right })
    };
}

macro_rules! gt {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::Gt, lhs: $left, rhs: $right })
    };
}
macro_rules! lt {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::Lt, lhs: $left, rhs: $right })
    };
}
macro_rules! or {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::Or, lhs: $left, rhs: $right })
    };
}
macro_rules! and {
    ($left:expr, $right:expr) => {
        Box::new(Expression::BinaryOp { op: BinOpType::And, lhs: $left, rhs: $right })
    };
}
macro_rules! vari {
    ($var:expr) => {
        Box::new(Expression::Variable { local: $var, idx: None })
    };
}
macro_rules! unbvari {
    ($var:expr) => {
        Expression::Variable { local: $var, idx: None }
    };
}
macro_rules! constant {
    ($value:expr) => {
        Box::new(Expression::Constant { value: $value.into() })
    };
}

pub fn handle_checked_op<'tcx, 'ccx>(
    fn_cx: &mut CodegenFunctionCx<'tcx, 'ccx>,
    op: BinOpType,
    lhs: Expression,
    rhs: Expression,
    ty: &Ty<'tcx>,
) -> Expression {
    let span = debug_span!("handle_checked_op").entered();
    // We can change the naming of the functions later
    let fn_name = format!("checked_{op}_{ty:?}");
    let fields = vec![rust_to_c_type(fn_cx.tcx, fn_cx.ongoing_codegen, ty), CType::Bool];
    let c_struct = fn_cx.ongoing_codegen.context.get_struct(&fields);
    if !fn_cx.ongoing_codegen.context.exists_header_fn_with_name(fn_name.as_str()) {
        warn!("Function for {fn_name} not found, creating one!");
        let checked_op = match op {
            BinOpType::Add => {
                if ty.is_signed() {
                    signed_add(&fn_name, &c_struct)
                } else {
                    unsigned_add(&fn_name, &c_struct)
                }
            }
            BinOpType::Sub => {
                if ty.is_signed() {
                    signed_sub(&fn_name, &c_struct)
                } else {
                    unsigned_sub(&fn_name, &c_struct)
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
    }
}

fn signed_add(fn_name: &String, c_struct: &CStruct) -> CFunction {
    let return_type = CType::Struct(CStructInfo::new(c_struct.get_name()));
    let sum_type = c_struct.get_field(0usize).get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(2, "sum".to_string(), sum_type.clone()), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(3, "overflow_a".to_string(), CType::Bool), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(4, "overflow_b".to_string(), CType::Bool), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(5, "overflow".to_string(), CType::Bool), None));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(assignment!(vari!(2), add!(vari!(0), vari!(1)))));

    //(second > 0 && first > INT32_MAX - second) || (second < 0 && first < INT32_MIN - second)
    bb.push(Statement::from_expression(assignment!(
        vari!(3),
        and!(gt!(vari!(1), constant!("0")), gt!(vari!(0), sub!(constant!("INT32_MAX"), vari!(1))))
    )));
    bb.push(Statement::from_expression(assignment!(
        vari!(4),
        and!(lt!(vari!(1), constant!("0")), lt!(vari!(0), sub!(constant!("INT32_MIN"), vari!(1))))
    )));
    bb.push(Statement::from_expression(assignment!(vari!(5), or!(vari!(3), vari!(4)))));
    bb.push(Statement::from_expression(Expression::Return {
        value: strct!(constant!(c_struct.get_name().clone()), vec![unbvari!(2), unbvari!(5)]),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn unsigned_add(fn_name: &String, c_struct: &CStruct) -> CFunction {
    let return_type = CType::Struct(CStructInfo::new(c_struct.get_name()));
    let sum_type = c_struct.get_field(0usize).get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), sum_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), sum_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(2, "sum".to_string(), sum_type.clone()), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(3, "overflow".to_string(), CType::Bool), None));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(assignment!(vari!(2), add!(vari!(0), vari!(1)))));
    // sum < first
    bb.push(Statement::from_expression(assignment!(vari!(3), lt!(vari!(2), vari!(0)))));
    bb.push(Statement::from_expression(Expression::Return {
        value: strct!(constant!(c_struct.get_name().clone()), vec![unbvari!(2), unbvari!(3)]),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn signed_sub(fn_name: &String, c_struct: &CStruct) -> CFunction {
    let return_type = CType::Struct(CStructInfo::new(c_struct.get_name()));
    let difference_type = c_struct.get_field(0usize).get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());
    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), difference_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), difference_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "difference".to_string(), difference_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(3, "overflow_a".to_string(), CType::Bool), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(4, "overflow_b".to_string(), CType::Bool), None));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(5, "overflow".to_string(), CType::Bool), None));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(assignment!(vari!(2), add!(vari!(0), vari!(1)))));

    //second > 0 && first < INT32_MIN + second
    bb.push(Statement::from_expression(assignment!(
        vari!(3),
        and!(gt!(vari!(1), constant!("0")), lt!(vari!(0), add!(constant!("INT32_MIN"), vari!(1))))
    )));
    // second < 0 && first > INT32_MAX + second
    bb.push(Statement::from_expression(assignment!(
        vari!(4),
        and!(lt!(vari!(1), constant!("0")), gt!(vari!(0), add!(constant!("INT32_MAX"), vari!(1))))
    )));
    //((second > 0 && first < INT32_MIN + second) || (second < 0 && first > INT32_MAX + second))
    bb.push(Statement::from_expression(assignment!(vari!(5), or!(vari!(3), vari!(4)))));
    bb.push(Statement::from_expression(Expression::Return {
        value: strct!(constant!(c_struct.get_name().clone()), vec![unbvari!(2), unbvari!(5)]),
    }));
    c_fn.push_bb(bb);
    c_fn
}

fn unsigned_sub(fn_name: &String, c_struct: &CStruct) -> CFunction {
    let return_type = CType::Struct(CStructInfo::new(c_struct.get_name()));
    let difference_type = c_struct.get_field(0usize).get_type();
    let mut c_fn = CFunction::new(fn_name.clone(), return_type.clone());

    c_fn.add_signature_var(CVarDef::new(0, "first".to_string(), difference_type.clone()));
    c_fn.add_signature_var(CVarDef::new(1, "second".to_string(), difference_type.clone()));
    c_fn.add_var_decl(CVarDecl::new(
        CVarDef::new(2, "difference".to_string(), difference_type.clone()),
        None,
    ));
    c_fn.add_var_decl(CVarDecl::new(CVarDef::new(3, "underflow".to_string(), CType::Bool), None));

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0));
    bb.push(Statement::from_expression(assignment!(vari!(2), sub!(vari!(0), vari!(1)))));
    // first < second
    bb.push(Statement::from_expression(assignment!(vari!(3), lt!(vari!(0), vari!(1)))));
    bb.push(Statement::from_expression(Expression::Return {
        value: strct!(constant!(c_struct.get_name().clone()), vec![unbvari!(2), unbvari!(3)]),
    }));
    c_fn.push_bb(bb);
    c_fn
}
