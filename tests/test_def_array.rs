#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_array
fn test_def_array() -> i32 {
    // CHECK: int32_t var1[3];
    let a = [1, 2, 3];
    // CHECK: int32_t var2[3];
    let b = [4, 5, 6];

    a[1] + b[1]
}
