#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_array() {
fn test_array() -> i32 {
    // CHECK: __WRAPPER_int32_t_3_ {{[a-zA-Z0-9_]+}};
    let mut a = [1, 2, 3];
    // CHECK: __WRAPPER_int32_t_3_ {{[a-zA-Z0-9_]+}};
    let b = [4, 5, 6];

    // CHECK: {{[a-zA-Z0-9_]+}}[{{[a-zA-Z0-9_]+}}] = 10;
    a[1] = 10;

    a[1] + b[1]
}
