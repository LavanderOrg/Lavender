use crate::_log;
use crate::debug;
use crate::info;
use crate::libs::arch;
use crate::libs::arch::paging::get_page_table_addr;
use crate::libs::arch::x86_64::LD_TEXT_START;
use crate::libs::arch::x86_64::memory::paging::PageEntryFlags;
use crate::libs::generic::memory;
use crate::libs::generic::memory::address::PhysAddr;
use crate::libs::generic::memory::address::VirtAddr;
use crate::libs::generic::memory::allocators::physical::bump::BumpAllocator;
use crate::libs::generic::memory::allocators::physical::pfa::PageFrameAllocator;
use crate::libs::generic::memory::paging::PageTable;
use crate::KERNEL_CONTEXT;
use limine::{memory_map::EntryType, response::MemoryMapResponse};

pub mod address;
pub mod paging;

pub mod allocators {
    pub mod physical {
        pub mod bump;
        pub mod pfa;
    }
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

    let mut top_pt = PageTable::new(get_page_table_addr(), crate::arch::paging::get_max_level());
    let ld_text_start = VirtAddr::try_from(&raw const LD_TEXT_START as u64).unwrap();
    let pte = top_pt.get_pte::<BumpAllocator>(ld_text_start, false, PageEntryFlags::all());

    debug!(
        "LD_TEXT_START Phys is at {:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical)
    );

    let mut kernel_pt = memory::paging::PageTable::new(arch::paging::get_page_table_addr(), arch::paging::get_max_level());

    kernel_pt.map_page::<BumpAllocator>(
        PhysAddr::try_from(&raw const LD_TEXT_START as u64).unwrap(),
        VirtAddr::try_from(0xffff6900000).unwrap(),
        PageEntryFlags::UserSupervisor);
    let pte = kernel_pt.get_pte::<BumpAllocator>(VirtAddr::try_from(0xffff6900000).unwrap(), false, PageEntryFlags::empty());

    debug!(
        "New LD_TEXT_START Phys is at {:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical)
    );
}
