#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_float
fn test_float() -> (f32, f64) {
    // CHECK: float {{[a-zA-Z0-9_]+}};
    let a = 1.0;
    // CHECK: double {{[a-zA-Z0-9_]+}};
    let b = 2.0;

    (a, b)
}
