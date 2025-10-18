use crate::_log;
use crate::debug;
use crate::libs::arch;
use crate::libs::arch::paging::get_page_level_size;
use crate::libs::arch::paging::get_page_table_addr;
use crate::libs::arch::x86_64::LD_TEXT_START;
use crate::libs::arch::x86_64::memory::paging::PageEntryFlags;
use crate::libs::generic::memory::address::PhysAddr;
use crate::libs::generic::memory::address::VirtAddr;
use crate::libs::generic::memory::allocators::physical::bump::BumpAllocator;
use crate::libs::generic::memory::allocators::physical::pfa::PageFrameAllocator;
use crate::libs::generic::memory::paging::PageTable;
use limine::{memory_map::EntryType, response::MemoryMapResponse};

pub mod address;
pub mod paging;

pub mod allocators {
    pub mod physical {
        pub mod bump;
        pub mod pfa;
    }
}

fn remap_kernel_section(new_pt: &mut PageTable, old_pt: &mut PageTable, section_address: VirtAddr, flags: PageEntryFlags) {
    let section_physical_addr: PhysAddr = unsafe {
        PhysAddr::try_from(
            old_pt.get_pte::<BumpAllocator>(
                section_address,
                false,
                PageEntryFlags::empty(),
            )
            .read()
            .get_address()
                + section_address.get_level_offset(paging::PaginationLevel::Physical),
        )
        .unwrap()
    };

    new_pt.map_page::<BumpAllocator>(
    section_physical_addr,
    section_address,
        flags | PageEntryFlags::UserSupervisor);
}

pub fn init(mmap: Option<&'static MemoryMapResponse>) {
    assert!(mmap.is_some());
    let entries: &[&limine::memory_map::Entry] = mmap.unwrap().entries();

    debug!("Memory map detection:");
    for entry in entries {
        if entry.entry_type == EntryType::RESERVED
            || entry.entry_type == EntryType::BOOTLOADER_RECLAIMABLE
        {
            continue;
        }
        _log!(
            "",
            "        [{:#x} - {:#x}] {} ({}MB)",
            entry.base,
            entry.base + entry.length,
            match entry.entry_type {
                EntryType::USABLE => "Free memory",
                EntryType::FRAMEBUFFER => "VESA Framebuffer",
                EntryType::EXECUTABLE_AND_MODULES => "Current kernel",
                EntryType::ACPI_NVS => "Reserved ACPI",
                EntryType::ACPI_RECLAIMABLE => "Reclaimable ACPI",
                EntryType::BAD_MEMORY => "Unusable memory (Bad or corrupted memory)",
                _ => "Unknown",
            },
            entry.length / 1024 / 1024
        );
    }

    unsafe {
        BumpAllocator::init(entries, crate::arch::paging::get_page_frame_size());
    }
    debug!(
        "Usable memory detected {}MiB",
        BumpAllocator::available_total() / 1024 / 1024
    );

    let mut bootloader_pte = PageTable::new(get_page_table_addr(), crate::arch::paging::get_max_level());
    let ld_text_start = VirtAddr::try_from(&raw const LD_TEXT_START as u64).unwrap();
    let pte = bootloader_pte.get_pte::<BumpAllocator>(ld_text_start, false, PageEntryFlags::all());

    debug!(
        "LD_TEXT_START Phys is at 0x{:02x}, Virt is at 0x{:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical), ld_text_start
    );

    let new_pte: PhysAddr = BumpAllocator::allocate_contiguous_range(get_page_level_size(), true);
    let mut kernel_pt: PageTable = PageTable::new(new_pte, arch::paging::get_max_level());

    remap_kernel_section(&mut kernel_pt ,&mut bootloader_pte, ld_text_start, PageEntryFlags::UserSupervisor);
    let pte: *mut paging::pmt::PageMapTableEntry = kernel_pt.get_pte::<BumpAllocator>(ld_text_start, false, PageEntryFlags::empty());

    debug!(
        "New LD_TEXT_START Phys is at {:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical)
    );
}
