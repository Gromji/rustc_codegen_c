#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_tuple
fn test_tuple() -> ((i32, i32), (i32, i32)) {
    // CHECK: t_int32_tint32_t {{[a-zA-Z0-9_]+}};
    let t1 = (1, 2);
    // CHECK: t_int32_tint32_t {{[a-zA-Z0-9_]+}};
    let t2 = (3, 4);

    (t1, t2)
}
