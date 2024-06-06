#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_char() {
fn test_char() -> (char, char) {
    // CHECK: char32_t {{[a-zA-Z0-9_]+}};
    let a = 'a';
    // CHECK: char32_t {{[a-zA-Z0-9_]+}};
    let b: char = 'b';

    (a, b)
}
