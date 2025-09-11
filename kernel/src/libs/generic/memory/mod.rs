use crate::_log;
use crate::debug;
use crate::info;
use crate::libs::arch::x86_64::LD_TEXT_START;
use crate::libs::arch::x86_64::memory::paging::paging::PageEntryFlags;
use crate::libs::arch::x86_64::registers::cr3;
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

pub fn init(mmap: Option<&'static MemoryMapResponse>) {
    assert!(mmap.is_some());
    let entries: &[&limine::memory_map::Entry] = mmap.unwrap().entries();

    info!("Memory map detection:");
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

    let mut pfa = BumpAllocator::new(entries, crate::arch::paging::get_page_frame_size());

    info!(
        "Usable memory detected {}MiB",
        pfa.available_total() / 1024 / 1024
    );
    for i in 0..2048 {
        let add = pfa.allocate(false);

        if i > 2045 {
            info!("PFA gave us 0x{:02x}", add);
        }
    }
    info!(
        "Remaining memory: {}MiB",
        (pfa.available_total() - pfa.used()) / 1024 / 1024
    );

    let mut top_pt = PageTable::new(cr3(), crate::arch::paging::get_max_level());
    let ld_text_start = VirtAddr::try_from(&raw const LD_TEXT_START as u64).unwrap();
    let pte = top_pt.get_pte(ld_text_start, false, PageEntryFlags::all());

    debug!(
        "LD_TEXT_START Phys is at {:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical)
    );
}
