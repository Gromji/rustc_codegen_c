#![crate_type = "lib"]

struct Strct {
    f1: i32,
    f2: i32,
    f3: u128,
    f4: bool,
    f5: char,
}

#[no_mangle]
// CHECK-LABEL: test_struct() {
fn test_struct() -> (i32, i32, u128, bool, char) {
    // CHECK: Strct {{[a-zA-Z0-9_]+}};
    let s = Strct {
        f1: 1,
        f2: 2,
        f3: 3,
        f4: true,
        f5: 'a',
    };

    (s.f1, s.f2, s.f3, s.f4, s.f5)
}
