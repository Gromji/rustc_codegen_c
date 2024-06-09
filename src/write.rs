use std::{fs::File, io::Write};

use crate::bb::BasicBlock;
use crate::bb::BasicBlockIdentifier;
use crate::definition::CVarDef;
use crate::expression::Expression;
use crate::function;
use crate::function::CFunction;
use crate::header;
use crate::include;
use crate::stmt::Statement;
use crate::structure;
use crate::ty::CIntTy;
use crate::ty::CType;

// Write includes to the file
pub fn write_includes(
    c_inc: &Vec<include::Include>,
    h_inc: &Vec<include::Include>,
    c_file: &mut File,
    h_file: &mut File,
) {
    let includes = c_inc.iter().map(|i| format!("{:?}", i)).collect::<Vec<String>>();
    let header_includes = h_inc.iter().map(|i| format!("{:?}", i)).collect::<Vec<String>>();

    c_file.write_all(includes.join("\n").as_bytes()).unwrap();
    h_file.write_all(header_includes.join("\n").as_bytes()).unwrap();
}

// Write Defines
pub fn write_defines(defines: &Vec<header::CDefine>, file: &mut File) {
    let defines = defines.iter().map(|d| format!("{:?}", d)).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(defines.join("\n").as_bytes()).unwrap();
}

// Write function prototypes
pub fn write_prototypes(functions: &Vec<function::CFunction>, file: &mut File) {
    let prototypes = functions.iter().map(|f| f.as_prototype()).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(prototypes.join("\n").as_bytes()).unwrap();
}

// Write structs to the file
pub fn write_structs(structs: &Vec<structure::CComposite>, file: &mut File) {
    let structs = structs.iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(structs.join("\n\n").as_bytes()).unwrap();
}

// Write the functions to the file
pub fn write_functions(functions: &mut Vec<function::CFunction>, file: &mut File, is_header: bool) {
    // Write newline
    file.write_all(b"\n\n").unwrap();

    let mut rust_main: Option<&mut CFunction> = None;
    let mut c_main: Option<&mut CFunction> = None;

    functions.iter_mut().for_each(|f| {
        if f.is_main() {
            rust_main = Some(f);
        } else {
            if f.get_name() == "main" {
                c_main = Some(f);
            }
        }
    });

    if !is_header {
        if let Some(c_m) = c_main {
            c_m.clear_bb();

            let mut bb = BasicBlock::new(BasicBlockIdentifier(0usize));

            if let Some(r_main) = rust_main {
                bb.push(Statement::from_expression(Expression::FnCall {
                    function: Box::new(Expression::Constant { value: r_main.get_name().into() }),
                    args: vec![],
                }));
            }

            bb.push(Statement::from_expression(Expression::Return {
                value: Box::new(Expression::Constant { value: "0".into() }),
            }));

            c_m.push_bb(bb);
        } else {
            let mut c_m = CFunction::new("main".to_string(), CType::Int(CIntTy::Int32));
            c_m.add_signature_var(CVarDef::new(0, "argc".to_string(), CType::Int(CIntTy::Int32)));
            c_m.add_signature_var(CVarDef::new(
                1,
                "argv".to_string(),
                CType::Array(Box::new(CType::Pointer(Box::new(CType::Int(CIntTy::Int8)))), 0),
            ));

            let mut bb = BasicBlock::new(BasicBlockIdentifier(0usize));

            if let Some(r_main) = rust_main {
                bb.push(Statement::from_expression(Expression::FnCall {
                    function: Box::new(Expression::Constant { value: r_main.get_name().into() }),
                    args: vec![],
                }));
            }

            bb.push(Statement::from_expression(Expression::Return {
                value: Box::new(Expression::Constant { value: "0".into() }),
            }));

            c_m.push_bb(bb);
            functions.push(c_m);
        }
    }

    let functions = functions.iter().map(|f| format!("{:?}", f)).collect::<Vec<String>>();
    file.write_all(functions.join("\n\n").as_bytes()).unwrap();
}
