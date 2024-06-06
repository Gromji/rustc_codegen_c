#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_bool() {
fn test_bool() -> (bool, bool) {
    // CHECK: bool {{[a-zA-Z0-9_]+}};
    let a = true;
    // CHECK: bool {{[a-zA-Z0-9_]+}};
    let b = false;

    (a, b)
}
