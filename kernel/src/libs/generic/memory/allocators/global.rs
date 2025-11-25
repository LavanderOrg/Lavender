use core::{alloc::{GlobalAlloc, Layout}, ffi::c_void};

use crate::libs::generic::memory::allocators::liballoc::{free, malloc};

struct Allocator {}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator {};

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = unsafe { self.alloc(layout) };

        if !ptr.is_null() {
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
        }

        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        let new_ptr = unsafe { self.alloc(new_layout) };

        if !new_ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
                self.dealloc(ptr, layout);
            }
        }
        new_ptr
    }

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { malloc(layout.padding_needed_for(layout.align()) + layout.size()) as *mut u8 }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        unsafe { free(ptr as *mut c_void); }
    }
}
