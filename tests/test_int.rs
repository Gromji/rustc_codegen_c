#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_signed_int
fn test_signed_int() -> (i8, i16, i32, i64, i128) {
    // CHECK: int8_t {{[a-zA-Z0-9_]+}};
    let b8: i8 = 0;
    // CHECK: int16_t {{[a-zA-Z0-9_]+}};
    let b16: i16 = 0;
    // CHECK: int32_t {{[a-zA-Z0-9_]+}};
    let b32: i32 = 0;
    // CHECK: int64_t {{[a-zA-Z0-9_]+}};
    let b64: i64 = 0;
    // CHECK: __int128_t {{[a-zA-Z0-9_]+}};
    let b128: i128 = 0;

    (b8, b16, b32, b64, b128)
}
