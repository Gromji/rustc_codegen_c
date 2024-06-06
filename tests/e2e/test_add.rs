#![crate_type = "lib"]
use std::arch::asm;

#[no_mangle]
unsafe fn add() -> i32 {
    let a = 5i32;
    let b = 6i32;

    custom_exit();

    a + b
}

#[no_mangle]
unsafe fn custom_exit() {
    asm!("mov rax, 60", "mov rdi, 1", "syscall");
}
