use crate::{
    KERNEL_CONTEXT, debug,
    libs::{
        arch::{
            paging::get_max_level,
            x86_64::{memory::paging::paging::PageEntryFlags, registers::cr3},
        },
        generic::memory::{
            address::*, allocators::physical::pfa::PageFrameAllocator,
            paging::pmt::PageMapTableEntry,
        },
    },
};

pub mod pmt;

#[derive(PartialEq, Debug)]
pub enum PaginationLevel {
    Physical = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
}
#[derive(Debug)]
pub struct UnsupportedPaginationLevel;

// TODO: Refactor, there's probably a way better way to do that lol.
impl TryFrom<u64> for PaginationLevel {
    type Error = UnsupportedPaginationLevel;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PaginationLevel::Physical),
            1 => Ok(PaginationLevel::Level1),
            2 => Ok(PaginationLevel::Level2),
            3 => Ok(PaginationLevel::Level3),
            4 => Ok(PaginationLevel::Level4),
            5 => Ok(PaginationLevel::Level5),
            _ => Err(UnsupportedPaginationLevel {}),
        }
    }
}

pub struct PageTable {
    pub head: u64,
    pub level: PaginationLevel,
}

impl PageTable {
    pub fn new(head: u64, level: PaginationLevel) -> Self {
        Self { head, level }
    }

    // TODO: Abstract, this is only valid for x86
    // TODO: Support different flags for each entry
    // Get the page table entry for a virtual address, if create is true
    // we create the leaf at each level until L0 (physical offset)
    pub fn get_pte(
        &mut self,
        virt_addr: VirtAddr,
        create: bool,
        flags: PageEntryFlags,
    ) -> *mut PageMapTableEntry {
        // Note: We're trying to go from the top level (5 in modern x86_64),
        // ensure that the level has an entry at the given address offset
        // if not, allocate one, and repeat for the next level.
        let mut head = self.head & 0xFFFFFFFFFF000 | unsafe { KERNEL_CONTEXT.boot_info.hhdm };

        for current_level in (2..(get_max_level() as u64 + 1)).rev() {
            unsafe {
                let pm_ptr: *mut u64 = (head) as *mut u64;
                let current_level_offset = virt_addr.get_level_offset(
                    PaginationLevel::try_from(current_level).expect("Unknown pagination level."),
                );
                let pm = PageMapTableEntry::from(*(pm_ptr.offset(current_level_offset as isize)));

                debug!(
                    "Address {:02x} ? Level {}, {}",
                    virt_addr, current_level, pm
                );
                if !pm.get_flags().contains(PageEntryFlags::Present) && create {
                    debug!(
                        "Allocating for address {:02x} as it is absent at level {}",
                        virt_addr, current_level
                    );
                }
                head =
                    pm.get_address() & 0xFFFFFFFFFF000 | unsafe { KERNEL_CONTEXT.boot_info.hhdm };
            }
        }
        head as *mut PageMapTableEntry
    }

    pub fn map_page(
        &mut self,
        allocator: &mut dyn PageFrameAllocator,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
    ) {
        
    }

    pub fn map_page_range(
        &mut self,
        allocator: &mut dyn PageFrameAllocator,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
    ) {
    }
}
