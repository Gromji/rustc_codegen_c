#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_signed_int
fn test_def_signed_int() -> (i8, i16, i32, i64, i128) {
    // CHECK: int8_t var1;
    let b8: i8 = 0;
    // CHECK: int16_t var2;
    let b16: i16 = 0;
    // CHECK: int32_t var3;
    let b32: i32 = 0;
    // CHECK: int64_t var4;
    let b64: i64 = 0;
    // CHECK: __int128_t var5;
    let b128: i128 = 0;

    (b8, b16, b32, b64, b128)
}
