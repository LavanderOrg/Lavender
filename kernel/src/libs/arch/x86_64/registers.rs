use core::arch::asm;

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
