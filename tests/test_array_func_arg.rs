#![crate_type = "lib"]

#[no_mangle]
fn test_array_func_arg(_arr: [[u8; 8]; 4]) -> [u8; 8] {
    [0; 8]
}
