# rustc_codegen_c

# Building the project
> make build

# Compiling .rs file
> make compile FILE_PATH=\<file-path\>

# Compiling .rs file and printing to stdout
> make compile_stdout FILE_PATH=\<file-path\>

# Running LLVM FileCheck tests
> make test

# Cleaning generated .c and .h files
> make clean_tests

# Environment variables

> RUST_LOG=\<debug | trace | info\> for controlling debug output

> C_CODEGEN_COMMENTS\=<EXCLUDE | INCLUDE\> for controlling auto generated comments 