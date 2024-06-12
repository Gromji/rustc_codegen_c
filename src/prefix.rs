use crate::base::Context;
use crate::bb::{BasicBlock, BasicBlockIdentifier};
use crate::definition::CVarDef;
use crate::fatptr::new_fat_ptr;
use crate::function::CFunction;

use crate::include::Include;
use crate::structure::CComposite;
use crate::ty::CIntTy;
use crate::ty::CType;

use crate::expression::Expression;
use crate::stmt::Statement;

pub fn build_prefix(context: &mut Context) {
    // Header Includes
    context
        .get_mut_h_includes()
        .append(&mut prefix_header_includes());
    // Functions
    context.get_mut_functions().append(&mut prefix_functions());
    // Structs
    context.get_mut_structs().append(&mut prefix_structs());
}

// Greedy list of includes
fn prefix_header_includes() -> Vec<Include> {
    vec![
        Include::new("stdio.h".to_string(), true),
        Include::new("stdint.h".to_string(), true),
        Include::new("stdlib.h".to_string(), true),
        Include::new("math.h".to_string(), true),
        Include::new("string.h".to_string(), true),
        Include::new("ctype.h".to_string(), true),
        Include::new("time.h".to_string(), true),
        Include::new("stddef.h".to_string(), true),
        Include::new("limits.h".to_string(), true),
        Include::new("float.h".to_string(), true),
        Include::new("stdbool.h".to_string(), true),
        Include::new("assert.h".to_string(), true),
        Include::new("errno.h".to_string(), true),
        Include::new("signal.h".to_string(), true),
        Include::new("setjmp.h".to_string(), true),
        Include::new("stdarg.h".to_string(), true),
        Include::new("locale.h".to_string(), true),
        Include::new("uchar.h".to_string(), true),
        Include::new("complex.h".to_string(), true),
    ]
}

// List of starter functions
fn prefix_functions() -> Vec<CFunction> {
    let mut functions: Vec<CFunction> = Vec::new();

    let mut main = CFunction::new("main".to_string(), CType::Int(CIntTy::Int32));

    main.add_signature_var(CVarDef::new(
        0,
        "argc".to_string(),
        CType::Int(CIntTy::Int32),
    ));
    main.add_signature_var(CVarDef::new(
        1,
        "argv".to_string(),
        CType::Array(
            Box::new(CType::Pointer(Box::new(CType::Int(CIntTy::Int8)))),
            0,
        ),
    ));

    // TODO: rewrite main.push("setlocale(LC_ALL, \"\");\n", false, 1);

    let mut bb = BasicBlock::new(BasicBlockIdentifier(0usize));
    bb.push(Statement::from_expression(Expression::Return {
        value: Box::new(Expression::Constant { value: "0".into() }),
    }));

    //return 0;
    main.push_bb(bb);

    functions.push(main);

    functions
}

// List of starter structs
fn prefix_structs() -> Vec<CComposite> {
    let structs: Vec<CComposite> = vec![new_fat_ptr()];
    structs
}
