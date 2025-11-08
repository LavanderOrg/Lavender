use limine::paging::Mode;

use crate::{
    debug, libs::{
        arch::{
            self, internal, paging::get_max_level, x86_64::memory::paging::{PageEntryFlags}
        },
        generic::memory::{
            address::*, allocators::physical::pfa::PageFrameAllocator,
            paging::pmt::PageMapTableEntry,
        },
    }
};

use num_traits::PrimInt;

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

pub type UnmappedAddressError = ();

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

impl TryFrom<PaginationLevel> for u64 {
    type Error = UnsupportedPaginationLevel;

    fn try_from(value: PaginationLevel) -> Result<Self, Self::Error> {
        match value {
            PaginationLevel::Physical => Ok(0),
            PaginationLevel::Level1 => Ok(1),
            PaginationLevel::Level2 => Ok(2),
            PaginationLevel::Level3 => Ok(3),
            PaginationLevel::Level4 => Ok(4),
            PaginationLevel::Level5 => Ok(5),
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

    pub fn dump(&self) {
        unsafe {
            for i in 0..512 {
                let pm_ptr: *mut PageMapTableEntry = self.head.as_hhdm().as_mut_ptr::<PageMapTableEntry>().offset(i as isize);
                let pm: PageMapTableEntry = *pm_ptr;

                if pm.get_flags().contains(PageEntryFlags::Present) && i == 0x1ff {
                    debug!("PMT L4 at {:02x} (index {:02x}): {}, bits: {}", pm_ptr.addr(), i, pm, pm.get_flags().bits());
                    for j in 0..512 {
                        let pm4: *mut PageMapTableEntry = ((pm.get_address().as_hhdm().as_mut_ptr()) as *mut PageMapTableEntry).offset(j);

                        if (*pm4).get_flags().contains(PageEntryFlags::Present) && j == 0x1fe {
                            debug!("     PDP at 0x{:02x} (index {:02x}): {}, bits {}", pm4.addr() as usize, j, *pm4, (*pm4).get_flags().bits());
                            for k in 0..512 {
                                let pdpe: *mut PageMapTableEntry = ((*pm4).get_address().as_hhdm().as_mut_ptr() as *mut PageMapTableEntry).offset(k);

                                if k == 0x00 && (*pdpe).get_flags().contains(PageEntryFlags::Present) {
                                    debug!("         PD at 0x{:02x} (index {:02x}): {}, bits {}", pdpe.addr(), k, *pdpe, (*pdpe).get_flags().bits());
                                    for l in 0..512 {
                                        let pde: *mut PageMapTableEntry = ((*pdpe).get_address().as_hhdm().as_mut_ptr() as *mut PageMapTableEntry).offset(l);

                                        if l == 0x1d && (*pde).get_flags().contains(PageEntryFlags::Present) {
                                            debug!("             PT at 0x{:02x}: {}, bits: {}", pde.addr(), *pde, (*pde).get_flags().bits());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // TODO: Abstract, this is only valid for x86
    // TODO: Support different flags for each entry
    // Get the page table entry for a virtual address, if create is true
    // we create the leaf at each level until L0 (physical offset)
    pub fn get_pte<P: PageFrameAllocator>(
        &mut self,
        virt_addr: VirtAddr,
        allocate: bool,
        flags: PageEntryFlags,
    ) -> Result<*mut PageMapTableEntry, UnmappedAddressError> {
        // Note: We're trying to go from the top level (5 in modern x86_64),
        // ensure that the level has an entry at the given address offset
        // if not, allocate one, and repeat for the next level until we reach PTE.
        let mut head: *mut PageMapTableEntry = unsafe { self.head.as_hhdm().as_mut_ptr() };

        // debug!("Top level paging address: 0x{:02x}", head);
        for current_level in (2..(get_max_level() as u64 + 1)).rev() {
            unsafe {
                let pm_ptr: *mut PageMapTableEntry = head as *mut PageMapTableEntry;
                let current_level_offset = virt_addr.get_level_offset(
                    PaginationLevel::try_from(current_level).expect("Unknown pagination level."),
                );
                let pm_offset_ptr: *mut PageMapTableEntry = pm_ptr.offset(current_level_offset as isize);

                if !(*pm_offset_ptr)
                    .get_flags()
                    .contains(PageEntryFlags::Present)
                {
                    if !allocate {
                        return Err(());
                    }
                    /*debug!(
                        "Allocating for address 0x{:02x} as it is absent at level {}",
                        virt_addr, current_level
                    );*/
                    let new_table_frame = P::allocate_contiguous_range(arch::paging::get_page_level_size(), true);

                    //debug!("New table frame at 0x{:02x}", new_table_frame);
                    (*pm_offset_ptr).set_address(new_table_frame.into());
                    (*pm_offset_ptr).set_flags(
                            PageEntryFlags::Present
                            | flags,
                    );
                }
                /*debug!(
                    "Address 0x{:02x} ? Head 0x{:02x} Level {}, 0x{:02x}",
                    virt_addr, head, current_level, (*pm_offset_ptr).get_address()
                );*/
                head = (*pm_offset_ptr).get_address().as_hhdm().as_mut_ptr::<PageMapTableEntry>();
            }
        }
        unsafe { Ok((head as *mut PageMapTableEntry).offset(virt_addr.get_level_offset(PaginationLevel::Level1) as isize)) }
    }

    pub fn map_page<P: PageFrameAllocator>(
        &mut self,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
        flags: PageEntryFlags,
    ) {
        let pte = self.get_pte::<P>(virt_addr, true, flags).unwrap();

        //debug!("Mapping phys 0x{:02x} to virt 0x{:02x} for PTE 0x{:02x}", phys_addr, virt_addr, pte.addr());
        unsafe {
            (*pte).set_address(phys_addr.into());
            (*pte).set_flags(flags | PageEntryFlags::Present);
        };
    }

    #[inline]
    pub fn align_up<T: PrimInt>(value: T, alignment: T) -> T {
        let mask = alignment - T::one();
        (value + mask) & !mask
    }

    pub fn map_page_range<P: PageFrameAllocator>(
        &mut self,
        phys_addr: PhysAddr,
        virt_addr: VirtAddr,
        flags: PageEntryFlags,
        length: usize
    ) {
        let length = Self::align_up(length, internal::memory::paging::get_page_frame_size());
        let step = internal::memory::paging::get_page_frame_size();

        debug!(
            "Mapping page range: phys 0x{:02x} to virt 0x{:02x}-0x{:02x}, length 0x{:02x}",
            phys_addr,
            virt_addr,
            virt_addr + length,
            length
        );
        for i in (0..length).step_by(step) {
            self.map_page::<P>(
    phys_addr + i,
    virt_addr + i,
            flags
            );
        }
    }
}
