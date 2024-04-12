use std::{fs::File, io::Write};

use crate::function;
use crate::prefix;

// Write the prefix code to the file
pub fn write_prefix(prefix: &prefix::Prefix, file: &mut File) {
    file.write_all(prefix.get_code().as_bytes()).unwrap();
}

// Write the functions to the file
pub fn write_functions(functions: &Vec<function::CFunction>, file: &mut File) {
    let main_exists = functions.iter().any(|f| f.is_main());
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
