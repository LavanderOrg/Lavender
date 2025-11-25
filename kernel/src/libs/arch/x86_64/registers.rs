use core::arch::asm;

pub fn cr0() -> u64 {
    let cr0: u64;

    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
    }

    cr0
}

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

pub fn cr4() -> u64 {
    let cr4: u64;

    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
    }

    cr4
}

pub fn write_cr0(value: u64) {
    unsafe {
        asm!("mov cr0, {addr}", addr = in(reg) (value));
    }
}

pub fn write_cr3(value: u64) {
    unsafe {
        asm!("mov cr3, {addr}", addr = in(reg) (value));
    }
}

pub fn write_cr4(value: u64) {
    unsafe {
        asm!("mov cr4, {addr}", addr = in(reg) (value));
    }
}

