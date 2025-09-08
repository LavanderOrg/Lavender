use bitflags::bitflags;
use limine::paging::Mode;

use crate::{KERNEL_CONTEXT, libs::generic::memory::paging::PaginationLevel};

bitflags!(
    pub struct PageEntryFlags: u64 {
        const Present = 1;
        const ReadWrite = 1 << 1;
        const UserSupervisor = 1 << 2;
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
