use std::{fs::File, io::Write};

use crate::crepr::Representable;
use crate::function;
use crate::header;
use crate::include;
use crate::structure;
use crate::alloc;

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

pub fn write_representables(reprs: &Vec<impl Representable>, file: &mut File) {
    let reprs = reprs.iter().map(|r| r.default_repr_str()).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(reprs.join("\n\n").as_bytes()).unwrap();
}

// Write Defines
pub fn write_defines(defines: &Vec<header::CDefine>, file: &mut File) {
    write_representables(defines, file)
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
    write_representables(structs, file)
}

// Write the functions to the file
pub fn write_functions(functions: &Vec<function::CFunction>, file: &mut File, is_header: bool) {
    let main_exists = functions.iter().any(|f| f.is_main());

    // Write newline
    file.write_all(b"\n\n").unwrap();

    if !main_exists && !is_header {
        file.write_all(
            b"int main(int argc, char* argv[]) {\
            \n  return 0;\
            \n}\
            \n",
        )
        .unwrap();
    }
    let functions = functions.iter().map(|f| format!("{:?}", f)).collect::<Vec<String>>();
    file.write_all(functions.join("\n\n").as_bytes()).unwrap();
}

pub fn write_statics(statics: &Vec<alloc::StaticAllocation>, file: &mut File) {
    write_representables(statics, file)
}