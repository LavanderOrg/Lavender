use super::internal;
use crate::libs::generic::memory::paging::PaginationLevel;

#[inline]
pub fn get_max_level() -> PaginationLevel {
    internal::memory::paging::paging::get_max_level()
}

#[inline]
pub fn get_page_frame_size() -> usize {
    internal::memory::paging::paging::get_page_frame_size()
}

#[inline]
pub fn enforce_canonical() -> bool {
    internal::memory::paging::paging::enforce_canonical()
}

