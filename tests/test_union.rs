#![crate_type = "lib"]

union TestUnion {
    a: i32,
    b: f32,
}

#[no_mangle]
// CHECK-LABEL: test_union() {
fn test_union() -> (i32, f32) {
    // CHECK: TestUnion {{[a-zA-Z0-9_]+}};
    let a = TestUnion { a: 1 };
    // CHECK: TestUnion {{[a-zA-Z0-9_]+}};
    let b = TestUnion { b: 1.0 };

    (unsafe { a.a }, unsafe { b.b })
}
