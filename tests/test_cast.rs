#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_cast() {
fn test_cast() -> usize {
    // CHECK: int32_t {{[a-zA-Z0-9_]+}};
    let num1 = 10i32;

    // CHECK: uint64_t {{[a-zA-Z0-9_]+}};
    let num2: usize;

    // CHECK: uint64_t {{[a-zA-Z0-9_]+}};
    let num3 = 20usize;

    // CHECK: (uint64_t){{[a-zA-Z0-9_]+}};
    num2 = num1 as usize;

    num2 + num3
}
