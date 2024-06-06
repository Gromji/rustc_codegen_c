#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_array() {
fn test_array() -> i32 {
    // CHECK: int32_t {{[a-zA-Z0-9_]+}}[3];
    let mut a = [1, 2, 3];
    // CHECK: int32_t {{[a-zA-Z0-9_]+}}[3];
    let b = [4, 5, 6];

    // CHECK: {{[a-zA-Z0-9_]+}}[{{[a-zA-Z0-9_]+}}] = 10;
    a[1] = 10;

    a[1] + b[1]
}
