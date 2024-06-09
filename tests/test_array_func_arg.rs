#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_array_func_arg(uint8_t var1[8][4]) {
fn test_array_func_arg(_arr: [[u8; 8]; 4]) -> [u8; 8] {
    [0; 8]
}
