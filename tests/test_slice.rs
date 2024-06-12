#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_slice() {
fn test_slice() -> &'static str {
    // CHECK: codegenc_fat_ptr {{[a-zA-Z0-9_]+}};
    let a = "Hello, world!";

    a
}
