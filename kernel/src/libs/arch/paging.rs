use super::internal;
use crate::libs::generic::memory::{address::PhysAddr, paging::PaginationLevel};

#[inline]
pub fn get_max_level() -> PaginationLevel {
    internal::memory::paging::get_max_level()
}

#[inline]
pub fn get_page_frame_size() -> usize {
    internal::memory::paging::get_page_frame_size()
}

#[inline]
pub fn enforce_canonical() -> bool {
    internal::memory::paging::enforce_canonical()
}

#[inline]
pub fn set_page_table_addr(addr: PhysAddr) {
    internal::memory::paging::set_page_table_addr(addr);
}

#[inline]
pub fn get_page_table_addr() -> PhysAddr {
    internal::memory::paging::get_page_table_addr()
}
