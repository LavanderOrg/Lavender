use crate::libs::arch::x86_64::{interrupts::ctx::Context, registers};
use core::arch::naked_asm;
use seq_macro::seq;

#[unsafe(no_mangle)]
pub extern "C" fn generic_handler(_context: *mut Context) {
    let context = unsafe { *_context };

    if context.isr_index == 0x1 {
        return;
    }
    match context.isr_index {
        0xE => {
            panic!(
                "Unhandled page fault occured while reading address 0x{:02x}\n\n{:?}{:?}",
                registers::cr2(),
                context,
                context.registers
            );
        }
        _ => panic!(
            "An unhandled CPU interrupt occured, {} (error code: {:x}, raw: {})\n\n{:?}{:?}",
            match context.isr_index {
                0x0 => "division by zero",
                0x2 => "NMI interrupt",
                0x3 => "breakpoint",
                0x4 => "overflow",
                0x5 => "bound range exceeded",
                0x6 => "invalid opcode",
                0x7 => "device not available (no math coprocessor)",
                0x9 => "coprocessor segment overrun",
                0x10 => "x87 floating point exception",
                0x12 => "machine check",
                0x13 => "SIMD floating point exception",
                0x14 => "virtualization exception",
                _ => "unknown error",
            },
            context.error_code,
            context.isr_index,
            context,
            context.registers
        ),
    }
}

// Note: There is no predefined macro to push all 15 general purpose registers (excluding RSP)
//       in x86_64 like PUSHAD so we make our own
//       to optimise slightly we decrement the RSP first and move each reg manually.
macro_rules! push_gpregs {
    () => {
        concat!(
            "sub rsp, 120;", // 15 * 8 bits
            "mov [rsp+112], rax;",
            "mov [rsp+104], rbx;",
            "mov [rsp+96], rcx;",
            "mov [rsp+88], rdx;",
            "mov [rsp+80], rbp;",
            "mov [rsp+72], rdi;",
            "mov [rsp+64], rsi;",
            "mov [rsp+56], r8;",
            "mov [rsp+48], r9;",
            "mov [rsp+40], r10;",
            "mov [rsp+32], r11;",
            "mov [rsp+24], r12;",
            "mov [rsp+16], r13;",
            "mov [rsp+8], r14;",
            "mov [rsp], r15;"
        )
    };
}

macro_rules! pop_gpregs {
    () => {
        concat!(
            "mov r15, [rsp];",
            "mov r14, [rsp+8];",
            "mov r13, [rsp+16];",
            "mov r12, [rsp+24];",
            "mov r11, [rsp+32];",
            "mov r10, [rsp+40];",
            "mov r9, [rsp+48];",
            "mov r8, [rsp+56];",
            "mov rsi, [rsp+64];",
            "mov rdi, [rsp+72];",
            "mov rbp, [rsp+80];",
            "mov rdx, [rsp+88];",
            "mov rcx, [rsp+96];",
            "mov rbx, [rsp+104];",
            "mov rax, [rsp+112];",
            "add rsp, 120;"
        )
    };
}

seq!(N in 0..=256 {
    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn isr_handler~N() {
        naked_asm!(
            ".if ({i} == 8 || ({i} >= 10 && {i} <= 14) || {i} == 17 || {i} == 21 || {i} == 29 || {i} == 30)",
            ".else",
                "push 0",
            ".endif",
            push_gpregs!(),
            "push {i}",
            "cld",
            "mov rdi, rsp",
            "call {generic_handler}",
            "add rsp, 8", // Pop ISR Index
            pop_gpregs!(),
            "add rsp, 8", // Pop error code
            "iretq",
            i = const N,
            generic_handler = sym generic_handler
        );
    }
});
