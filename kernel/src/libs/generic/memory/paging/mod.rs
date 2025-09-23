use limine::paging::Mode;

use crate::{
    debug, libs::{
        arch::{
            self, paging::get_max_level, x86_64::{memory::paging::PageEntryFlags}
        },
        generic::memory::{
            address::*, allocators::physical::pfa::PageFrameAllocator,
            paging::pmt::PageMapTableEntry,
        },
    }, KERNEL_CONTEXT
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

impl TryFrom<Mode> for PaginationLevel {
    type Error = UnsupportedPaginationLevel;

    fn try_from(value: Mode) -> Result<Self, Self::Error> {
        match value {
            Mode::FOUR_LEVEL => Ok(PaginationLevel::Level4),
            Mode::FIVE_LEVEL => Ok(PaginationLevel::Level5),
            _ => Err(UnsupportedPaginationLevel {}),
        }
    }
}

pub struct PageTable {
    pub head: PhysAddr,
    pub level: PaginationLevel,
}

impl PageTable {
    pub fn new(head: PhysAddr, level: PaginationLevel) -> Self {
        Self { head, level }
    }

    pub fn load(&self) {
        arch::paging::set_page_table_addr(self.head);
    }

    // TODO: Abstract, this is only valid for x86
    // TODO: Support different flags for each entry
    // Get the page table entry for a virtual address, if create is true
    // we create the leaf at each level until L0 (physical offset)
    pub fn get_pte<P: PageFrameAllocator>(
        &mut self,
        virt_addr: VirtAddr,
        create: bool,
        flags: PageEntryFlags,
    ) -> *mut PageMapTableEntry {
        // Note: We're trying to go from the top level (5 in modern x86_64),
        // ensure that the level has an entry at the given address offset
        // if not, allocate one, and repeat for the next level until we reach PTE.
        let mut head: u64 = Into::<u64>::into(self.head) | unsafe { KERNEL_CONTEXT.boot_info.hhdm };

        for current_level in (2..(get_max_level() as u64 + 1)).rev() {
            unsafe {
                let pm_ptr: *mut PageMapTableEntry = head as *mut PageMapTableEntry;
                let current_level_offset = virt_addr.get_level_offset(
                    PaginationLevel::try_from(current_level).expect("Unknown pagination level."),
                );
                let pm_offset_ptr: *mut PageMapTableEntry = pm_ptr.offset(current_level_offset as isize);

                debug!(
                    "Address {:02x} ? Level {}, {}",
                    virt_addr, current_level, *pm_offset_ptr
                );
                if !(*pm_offset_ptr)
                    .get_flags()
                    .contains(PageEntryFlags::Present)
                    && create
                {
                    debug!(
                        "Allocating for address {:02x} as it is absent at level {}",
                        virt_addr, current_level
                    );
                    /*let new_table_frame = P::allocate_contiguous_range(arch::paging::get_page_level_size(), true);

                    (*pm_offset_ptr).set_address(new_table_frame.into());
                    (*pm_offset_ptr).set_flags(
                            PageEntryFlags::Present
                            | PageEntryFlags::ReadWrite
                            | flags,
                    );*/
                }
                head = (*pm_offset_ptr).get_address()
                    | unsafe { KERNEL_CONTEXT.boot_info.hhdm };
            }
        }
        head as *mut PageMapTableEntry
    }

    pub fn map_page<P: PageFrameAllocator>(
        &mut self,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
        flags: PageEntryFlags,
    ) {
        let pte = self.get_pte::<P>(virt_addr, true, flags);

        unsafe { (*pte).set_address(phys_addr.into()) };
    }

    pub fn map_page_range<P: PageFrameAllocator>(
        &mut self,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
    ) {
    }
}
