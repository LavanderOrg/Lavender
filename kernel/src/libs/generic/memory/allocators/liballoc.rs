use core::ffi::c_void;

use crate::{debug, libs::generic::memory::allocators::physical::{bump::BumpAllocator, pfa::PageFrameAllocator}};

#[link(name = "alloc", kind = "static")]
unsafe extern "C" {
    pub unsafe fn malloc(size: usize) -> *mut c_void;
    pub unsafe fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    pub unsafe fn free(ptr: *mut c_void);
}

#[unsafe(no_mangle)]
pub extern "C" fn liballoc_lock() {
    // debug!("Locking liballoc");
}

#[unsafe(no_mangle)]
pub extern "C" fn liballoc_unlock() {
    // debug!("Unlocking liballoc");
}

#[unsafe(no_mangle)]
pub extern "C" fn liballoc_free(_ptr: *mut c_void, _: i32) {
    debug!("Freeing memory at {:p}", _ptr);
}

#[unsafe(no_mangle)]
pub extern "C" fn liballoc_alloc(pages_num: i32) -> *mut c_void {
    //debug!("Allocating {} pages of memory", pages_num);
    let head = BumpAllocator::allocate_contiguous_range(crate::libs::arch::paging::get_page_frame_size() * pages_num as usize, false).as_hhdm().into();

    //debug!("Allocated memory at {:p}", head);
    return head;
}
