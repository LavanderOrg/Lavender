use crate::{debug, libs::arch::x86_64::cpu::CpuIdRequest};

/*
Note: I try to put most inline ASM in this file so that I can review it later
      and understand why it's ugly.
*/
use super::gdt::GdtDescriptor;
use core::arch::asm;

#[inline]
pub unsafe fn cli() {
    unsafe {
        asm!("cli");
    }
}

#[inline]
pub unsafe fn outb(port: usize, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nostack));
    }
}

#[inline]
pub unsafe fn inb(port: usize) -> u8 {
    unsafe {
        let ret: u8;

        asm!("in al, dx", in("dx") port, out("al") ret , options(nostack));
        ret
    }
}

#[inline]
pub unsafe fn load_gdt(gdtr: &GdtDescriptor) {
    debug!("Loading GDT at address {:02x}", gdtr.gdt as u64);
    unsafe {
        asm!(
            "cli",
            "lgdt [{gdtr}]",
            "push {codeseg_offset}",
            "lea RAX, [rip + 2f]",
            "push RAX",
            "retfq",
        "2:",
            "mov ax, {dataseg_offset}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            gdtr = in(reg) gdtr,
            // Ideally you would get the offsets dynamically from the Gdtr, but we convert to u64 early
            // making it impossible.
            // TODO: Refactor
            codeseg_offset = const 0x8,
            dataseg_offset = const 0x10,
            lateout("rax") _
        );
    }
}

pub enum CpuIdRegisterOrder {
    EAX = 0,
    EBX = 1,
    ECX = 2,
    EDX = 3,
}

#[inline]
pub unsafe fn cpuid(request: CpuIdRequest) -> [u32; 4] {
    let mut result: [u32; 4] = [0; 4];

    unsafe {
        asm!(
            "mov {0:r}, rbx",
            "mov rax, 1",
            "mov rcx, 0", // TODO: Add support for subleaves
            "cpuid",
            "xchg {0:r}, rbx",
            out(reg) result[1],
            lateout("eax") result[0],
            lateout("ecx") result[2],
            lateout("edx") result[3],
            in("eax") request as u32,
            options(nostack)
        );
    }
    result
}
