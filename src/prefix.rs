use crate::base::Context;
use crate::definition::CVarDef;
use crate::function::CFunction;

use crate::include::Include;
use crate::structure::CStruct;
use crate::ty::CType;
use crate::ty::CIntTy;

pub fn build_prefix(context: &mut Context) {
    // Includes
    context.get_mut_includes().append(&mut prefix_includes());
    // Functions
    context.get_mut_functions().append(&mut prefix_functions());
    // Structs
    context.get_mut_structs().append(&mut prefix_structs());
}

// Greedy list of includes
fn prefix_includes() -> Vec<Include> {
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
        Include::new("wchar.h".to_string(), true),
        Include::new("complex.h".to_string(), true),
    ]
}

// List of starter functions
fn prefix_functions() -> Vec<CFunction> {
    let mut functions: Vec<CFunction> = Vec::new();

    let mut main = CFunction::new("main".to_string(), CType::Int(CIntTy::Int32));

    main.add_signature_var(CVarDef::new("argc".to_string(), CType::Int(CIntTy::Int32)));
    main.add_signature_var(CVarDef::new(
        "argv".to_string(),
        CType::Array(Box::new(CType::Pointer(Box::new(CType::Int(CIntTy::Int8)))), 0),
    ));
    main.push("setlocale(LC_ALL, \"\");\n", false, 1);
    main.push("return 0;\n", false, 1);

    functions.push(main);

    functions
}

// List of starter structs
fn prefix_structs() -> Vec<CStruct> {
    // add an unit struct
    let mut structs: Vec<CStruct> = Vec::new();
    let mut unit_struct = CStruct::new("__Unit".to_string());
    structs.push(unit_struct);

    structs
}
