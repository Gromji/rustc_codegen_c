#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_char
fn test_def_char() -> (char, char) {
    // CHECK: char32_t var1;
    let a = 'a';
    // CHECK: char32_t var2;
    let b: char = 'b';

    (a, b)
}
