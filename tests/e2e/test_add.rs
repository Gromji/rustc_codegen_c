#![crate_type = "lib"]

#[no_mangle]
fn add() -> i32 {
    let a = 5i32;
    let b = 6i32;

    a + b
}
