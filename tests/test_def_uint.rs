#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_def_unsigned_int
fn test_def_unsigned_int() -> (u8, u16, u32, u64, u128) {
    // CHECK: uint8_t var1;
    let b8: u8 = 0;
    // CHECK: uint16_t var2;
    let b16: u16 = 0;
    // CHECK: uint32_t var3;
    let b32: u32 = 0;
    // CHECK: uint64_t var4;
    let b64: u64 = 0;
    // CHECK: __uint128_t var5;
    let b128: u128 = 0;

    (b8, b16, b32, b64, b128)
}
