use core::arch::asm;

use crate::debug;

pub fn cr2() -> u64 {
    let cr2: u64;

    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }

    cr2
}

pub fn cr3() -> u64 {
    let cr3: u64;

    unsafe {
        asm!("mov {}, cr3", out(reg) cr3);
    }

    cr3
}

pub fn write_cr3(value: u64) {
    unsafe {
        asm!("mov cr3, {addr}", addr = in(reg) (value));
    }
}
