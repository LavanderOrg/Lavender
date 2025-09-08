use limine::memory_map::EntryType;

use crate::libs::generic::memory::{
    address::PhysAddr, allocators::physical::pfa::PageFrameAllocator,
};

pub struct BumpAllocator {
    // TODO: Refactor, I don't want limine references in the kernel after init.
    memory_map: &'static [&'static limine::memory_map::Entry],
    pfsize: usize,
    head: usize,
}

impl BumpAllocator {
    pub fn new(memory_map: &'static [&limine::memory_map::Entry], pfsize: usize) -> Self {
        Self {
            memory_map,
            pfsize,
            head: 0,
        }
    }

    fn mem_iter(&self) -> impl Iterator<Item = u64> {
        self.memory_map
            .iter()
            .filter(|x| {
                x.entry_type == EntryType::USABLE
                    && x.length >= self.pfsize as u64
                    && x.base > (1 << 16)
            })
            .map(|x| x.base..(x.base + x.length))
            .flat_map(|x| x.step_by(self.pfsize))
    }
}

impl PageFrameAllocator for BumpAllocator {
    fn allocate(&mut self, clear: bool) -> PhysAddr {
        let head = PhysAddr::from(
            self.mem_iter()
                .nth(self.head)
                .expect("Page frame allocator is out of usable memory."),
        );

        self.head += 1;
        if clear {
            unsafe {
                core::ptr::write_bytes(head.as_hhdm().into(), 0, self.pfsize);
            }
        }
        head
    }

    fn free(&mut self) {
        panic!("Cannot call free() on a bump allocator.");
    }

    fn available_total(&self) -> usize {
        self.mem_iter().count() * self.pfsize
    }

    fn used(&self) -> usize {
        self.head * self.pfsize
    }
}
