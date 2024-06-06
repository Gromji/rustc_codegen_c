#![crate_type = "lib"]

enum TestEnum {
    A,
    B,
}

#[no_mangle]
// CHECK-LABEL: test_enum() {
fn test_enum() -> (bool, bool) {
    // CHECK: TestEnum {{[a-zA-Z0-9_]+}};
    let a = TestEnum::A;
    // CHECK: TestEnum {{[a-zA-Z0-9_]+}};
    let b = TestEnum::B;

    (matches!(a, TestEnum::A), matches!(b, TestEnum::B))
}
