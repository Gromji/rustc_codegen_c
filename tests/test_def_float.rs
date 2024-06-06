#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_float
fn test_def_float() -> (f32, f64) {
    // CHECK: float var1;
    let a = 1.0;
    // CHECK: double var2;
    let b = 2.0;

    (a, b)
}
