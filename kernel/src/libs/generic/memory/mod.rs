use crate::_log;
use crate::debug;
use crate::libs::arch;
use crate::libs::arch::paging::get_page_level_size;
use crate::libs::arch::paging::get_page_table_addr;
use crate::libs::arch::x86_64::LD_DATA_END;
use crate::libs::arch::x86_64::LD_DATA_START;
use crate::libs::arch::x86_64::LD_RODATA_END;
use crate::libs::arch::x86_64::LD_RODATA_START;
use crate::libs::arch::x86_64::LD_TEXT_END;
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

fn remap_kernel_section(new_pt: &mut PageTable, old_pt: &mut PageTable, section_address_start: VirtAddr, section_address_end: VirtAddr, flags: PageEntryFlags) {
    let pte = unsafe {
        old_pt.get_pte::<BumpAllocator>(
        section_address_start,
        false,
        PageEntryFlags::empty(),
        ).unwrap().read() };
    let offset = section_address_start.get_level_offset(paging::PaginationLevel::Physical);
    let section_physical_addr: PhysAddr = unsafe {
        PhysAddr::try_from(pte.get_address() + offset as usize).unwrap()
    };

    debug!(
        "Remapping kernel section from phys 0x{:02x} to virt 0x{:02x}",
        section_physical_addr,
        section_address_start
    );
    new_pt.map_page_range::<BumpAllocator>(
        section_physical_addr,
        section_address_start,
        flags,
            (section_address_end - section_address_start).into()
    );
}

pub fn init(mmap: Option<&'static MemoryMapResponse>) {
    assert!(mmap.is_some());
    let entries: &[&limine::memory_map::Entry] = mmap.unwrap().entries();

    debug!("Memory map detection:");
    for entry in entries {
        if entry.entry_type == EntryType::RESERVED
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
                EntryType::BOOTLOADER_RECLAIMABLE => "Reclaimable bootloader",
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
    debug!("Bootloader page table: 0x{:02x}", get_page_table_addr());
    bootloader_pte.dump();
    let new_pt: PhysAddr = BumpAllocator::allocate_contiguous_range(get_page_level_size(), true);
    debug!("New page table allocated at phys 0x{:02x}", new_pt);
    let sections: [(u64, u64, PageEntryFlags); 3] = [
        (&raw const LD_TEXT_START as u64, &raw const LD_TEXT_END as u64, PageEntryFlags::empty() ),
        (&raw const LD_RODATA_START as u64, &raw const LD_RODATA_END as u64, PageEntryFlags::ExecuteDisabled),
        (&raw const LD_DATA_START as u64, &raw const LD_DATA_END as u64, PageEntryFlags::ExecuteDisabled | PageEntryFlags::ReadWrite),
    ];
    let mut kernel_pt: PageTable = PageTable::new(new_pt, arch::paging::get_max_level());

    unsafe { VirtAddr::try_from(&raw const LD_TEXT_START as u64).unwrap().dump_offsets() };
    for section in sections {
        let section_start: VirtAddr = VirtAddr::try_from(section.0).unwrap();
        let section_end: VirtAddr = VirtAddr::try_from(section.1).unwrap();

        remap_kernel_section(
            &mut kernel_pt,
            &mut bootloader_pte,
            section_start,
            section_end, section.2);
        //kernel_pt.dump();
        let pte: *mut paging::pmt::PageMapTableEntry = kernel_pt.get_pte::<BumpAllocator>(section_start, false, PageEntryFlags::all()).unwrap();

        debug!(
            "New section phys is at 0x{:02x}, Virt is at 0x{:02x}",
            unsafe { pte.read().get_address() + section_start.get_level_offset(paging::PaginationLevel::Physical) as usize}, section_start
        );
    }


    debug!("Remapped kernel sections.");
    entries.iter()
        .filter(|entry|
            entry.entry_type == EntryType::USABLE ||
            entry.entry_type == EntryType::ACPI_RECLAIMABLE ||
            entry.entry_type == EntryType::BOOTLOADER_RECLAIMABLE)
        .for_each(|section| {
            kernel_pt.map_page_range::<BumpAllocator>(
                section.base.into(),
                PhysAddr::from(section.base).as_hhdm(),
                PageEntryFlags::ReadWrite,
                section.length as usize);
        });

    kernel_pt.dump();

    let cr2: VirtAddr = VirtAddr::try_from(0xffff80007a39f132).unwrap();
    let pte_old = bootloader_pte.get_pte::<BumpAllocator>(cr2, false, PageEntryFlags::empty());
    let pte_new = kernel_pt.get_pte::<BumpAllocator>(cr2, false, PageEntryFlags::empty());

    unsafe { cr2.dump_offsets() };
    debug!("CR2 Address on OLD PT ? {}, {:02x}", pte_old.is_ok(), pte_old.unwrap().read().get_address() + cr2.get_level_offset(paging::PaginationLevel::Physical) as usize);
    debug!("CR2 Address on NEW PT ? {}, {:02x}", pte_new.is_ok(), pte_new.unwrap().read().get_address() + cr2.get_level_offset(paging::PaginationLevel::Physical) as usize);
    kernel_pt.load();
    debug!("Loaded new PT.");
    while 1 == 1 {

    }



    /*

    debug!(
        "LD_TEXT_START Phys is at 0x{:02x}, Virt is at 0x{:02x}",
        unsafe { pte.read().get_address() }
            + ld_text_start.get_level_offset(paging::PaginationLevel::Physical), ld_text_start
    );



    remap_kernel_section(&mut kernel_pt ,&mut bootloader_pte, ld_text_start, PageEntryFlags::UserSupervisor);
    let pte: *mut paging::pmt::PageMapTableEntry = kernel_pt.get_pte::<BumpAllocator>(ld_text_start, false, PageEntryFlags::empty());

    */
}
