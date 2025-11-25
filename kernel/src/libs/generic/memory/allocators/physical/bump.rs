use limine::memory_map::EntryType;

use crate::libs::{arch, generic::memory::{
    address::PhysAddr, allocators::physical::pfa::PageFrameAllocator,
}};

pub struct BumpAllocatorState {
    memory_map: &'static [&'static limine::memory_map::Entry],
    pfsize: usize,
    head: usize,
}

static mut STATE: BumpAllocatorState = BumpAllocatorState {
    memory_map: &[],
    pfsize: 0,
    head: 0,
};

pub struct BumpAllocator {}
impl BumpAllocator {
    pub fn init(memory_map: &'static [&limine::memory_map::Entry], pfsize: usize) {
        unsafe {
            STATE.memory_map = memory_map;
            STATE.pfsize = pfsize;
            STATE.head = 0;
        }
    }

    fn mem_iter() -> impl Iterator<Item = u64> {
        unsafe { STATE.memory_map
            .iter()
            .filter(|x| {
                x.entry_type == EntryType::USABLE
                    && x.length >= STATE.pfsize as u64
                    && x.base > (1 << 16)
            })
            .map(|x| x.base..(x.base + x.length))
            .flat_map(|x| x.step_by(STATE.pfsize as usize)) }
    }
}

impl PageFrameAllocator for BumpAllocator {
    fn allocate(clear: bool) -> PhysAddr {
        let head = PhysAddr::from(
            BumpAllocator::mem_iter()
                .nth(unsafe { STATE.head as usize })
                .expect("Page frame allocator is out of usable memory."),
        );

        unsafe {
            STATE.head += 1;
            if clear {
                unsafe {
                    core::ptr::write_bytes(Into::<*mut u8>::into(head.as_hhdm()), 0, STATE.pfsize as usize);
                }
            }
        }
        return head;
    }

    fn free() {
        panic!("Cannot call free() on a bump allocator.");
    }

    fn available_total() -> usize {
        BumpAllocator::mem_iter().count() * unsafe { STATE.pfsize as usize }
    }

    fn used() -> usize {
        unsafe {
            STATE.head as usize * STATE.pfsize as usize
        }
    }

    fn allocate_contiguous_range(size: usize, clear: bool) -> PhysAddr {
        let mut total_size = size;

        if total_size == 0 {
            total_size = arch::paging::get_page_frame_size();
        }
        let pages = total_size / arch::paging::get_page_frame_size() + 1;
        let head = BumpAllocator::allocate(clear);

        for _ in 1..pages {
            BumpAllocator::allocate(clear);
        }

        head
    }
}
