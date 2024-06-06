#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_bool
fn test_def_bool() -> (bool, bool) {
    // CHECK: bool var1;
    let a = true;
    // CHECK: bool var2;
    let b = false;

    (a, b)
}
