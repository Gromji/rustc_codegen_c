use std::{fs::File, io::Write};

use crate::function;
use crate::include;
use crate::structure;

// Write includes to the file
pub fn write_includes(includes: &Vec<include::Include>, file: &mut File) {
    let includes = includes.iter().map(|i| i.to_string()).collect::<Vec<String>>();

    file.write_all(includes.join("\n").as_bytes()).unwrap();
}

// Write function prototypes
pub fn write_prototypes(functions: &Vec<function::CFunction>, file: &mut File) {
    let prototypes = functions.iter().map(|f| f.as_prototype()).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(prototypes.join("\n").as_bytes()).unwrap();
}

// Write structs to the file
pub fn write_structs(structs: &Vec<structure::CStruct>, file: &mut File) {
    let structs = structs.iter().map(|s| s.to_string()).collect::<Vec<String>>();

    // Write newline
    file.write_all(b"\n\n").unwrap();
    file.write_all(structs.join("\n\n").as_bytes()).unwrap();
}

// Write the functions to the file
pub fn write_functions(functions: &Vec<function::CFunction>, file: &mut File) {
    let main_exists = functions.iter().any(|f| f.is_main());

    // Write newline
    file.write_all(b"\n\n").unwrap();

    if !main_exists {
        file.write_all(
            b"int main(int argc, char* argv[]) {\
            \n  return 0;\
            \n}\
            \n",
        )
        .unwrap();
    }
    let functions = functions.iter().map(|f| f.to_string()).collect::<Vec<String>>();
    file.write_all(functions.join("\n\n").as_bytes()).unwrap();
}
