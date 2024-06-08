#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_array_func_arg() {
fn test_array_func_arg(_arr: [[u8; 4]; 4]) -> [u8; 4] {
    [0; 4]
}
