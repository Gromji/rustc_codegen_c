#![crate_type = "lib"]

#[no_mangle]
// CHECK-LABEL: test_unsigned_int() {
fn test_unsigned_int() -> (u8, u16, u32, u64, u128) {
    // CHECK: uint8_t {{[a-zA-Z0-9_]+}};
    let b8: u8 = 0;
    // CHECK: uint16_t {{[a-zA-Z0-9_]+}};
    let b16: u16 = 0;
    // CHECK: uint32_t {{[a-zA-Z0-9_]+}};
    let b32: u32 = 0;
    // CHECK: uint64_t {{[a-zA-Z0-9_]+}};
    let b64: u64 = 0;
    // CHECK: __uint128_t {{[a-zA-Z0-9_]+}};
    let b128: u128 = 0;

    (b8, b16, b32, b64, b128)
}
