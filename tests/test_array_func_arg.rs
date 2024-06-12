#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_array_func_arg(__WRAPPER___WRAPPER_uint8_t_8__4_ var1) {
fn test_array_func_arg(_arr: [[u8; 8]; 4]) -> [u8; 8] {
    [0; 8]
}
