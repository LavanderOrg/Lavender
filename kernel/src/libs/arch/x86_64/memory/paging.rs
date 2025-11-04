use bitflags::bitflags;
use limine::paging::Mode;

use crate::{libs::{arch::x86_64::registers::{cr3, write_cr3}, generic::memory::{address::PhysAddr, paging::PaginationLevel}}, KERNEL_CONTEXT};

bitflags!(
    #[derive(Copy, Clone)]
    pub struct PageEntryFlags: u64 {
        const Present = 1;
        const ReadWrite = 1 << 1;
        const User = 1 << 2;
        const WriteThrough = 1 << 3;
        const CacheDisabled = 1 << 4;
        const Accessed = 1 << 5;
        const Dirty = 1 << 6;
        const PageAttributeTable = 1 << 7;
        const Global = 1 << 8;
        const ExecuteDisabled = 1 << 63;
    }
);

pub const ADDRESS_MASK: u64 = 0xFFFFFFFFFF000;

pub fn get_max_level() -> PaginationLevel {
    match unsafe {
        KERNEL_CONTEXT
            .boot_info
            .paging_level
            .expect("Couldn't read BOOTINFO structure for max paging level.")
    } {
        Mode::FIVE_LEVEL => PaginationLevel::Level5,
        Mode::FOUR_LEVEL => PaginationLevel::Level4,
        _ => PaginationLevel::Level3,
    }
}

#[inline]
pub fn get_page_frame_size() -> usize {
    4096
}

#[inline]
pub fn enforce_canonical() -> bool {
    true
}

#[inline]
pub fn set_page_table_addr(addr: PhysAddr) {
    write_cr3((cr3() & 0xFFF) | Into::<u64>::into(addr));
}

#[inline]
pub fn get_page_table_addr() -> PhysAddr {
    PhysAddr::from(cr3() & ADDRESS_MASK)
}

#[inline]
pub fn get_page_level_size() -> usize {
    256 * 64
}
