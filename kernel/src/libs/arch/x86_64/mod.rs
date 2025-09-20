use seq_macro::seq;

use crate::KERNEL_CONTEXT;
use crate::libs::arch::x86_64::cpu::CpuInfo;
use crate::libs::arch::x86_64::memory::paging::PageEntryFlags;
use crate::libs::generic::memory::paging::pmt::PageMapTableEntry;
use crate::{
    info,
    libs::arch::x86_64::{
        gdt::{CPL_RING_0, SegmentSelector},
        interrupts::idt::{Idt, IdtDescriptor, IdtGateDescriptor, IdtGateDescriptorProperties},
    },
};
use core::arch::asm;

pub mod asm;
pub mod cpu;
pub mod gdt;
pub mod memory;
pub mod registers;
pub mod interrupts {
    pub mod ctx;
    pub mod idt;
    pub mod isr;
}

struct CpuContext {
    gdt: [u64; 5],
    idtr: Option<IdtDescriptor>,
    info: Option<CpuInfo>,
}

// NOTE: Yeah buddy you'll have to modify some of that for multi-proc support innit bruv
static mut CPU_CONTEXT: CpuContext = CpuContext {
    gdt: [0, 0, 0, 0, 0],
    idtr: None,
    info: None,
};

fn dump_page_tables() {
    unsafe {
        let mut cr3 = registers::cr3();

        info!("CR3 value: {:02x}", cr3);
        cr3 = cr3 & 0xFFFFFFFFFF000;
        for i in 0..512 {
            let pm_ptr: *mut u64 = (cr3 | KERNEL_CONTEXT.boot_info.hhdm) as *mut u64;
            let pm = PageMapTableEntry::from(*(pm_ptr.offset(i as isize)));

            if pm.get_flags().contains(PageEntryFlags::Present) {
                info!("PMT L5 at {:02x} (index {}): {}", cr3, i, pm);

                let pm4_ptr: *mut u64 =
                    (pm.get_address() | KERNEL_CONTEXT.boot_info.hhdm) as *mut u64;
                let pm4 = PageMapTableEntry::from(*(pm4_ptr));

                info!("     PMT L4 at 0x{:02x}: {}", pm4_ptr as usize, pm4);
                if pm4.get_flags().contains(PageEntryFlags::Present) {
                    let pdpe_ptr: *mut u64 =
                        (pm4.get_address() | KERNEL_CONTEXT.boot_info.hhdm) as *mut u64;
                    let pdpe = PageMapTableEntry::from(*(pdpe_ptr));

                    info!("         PDPE at 0x{:02x}: {}", pdpe_ptr as usize, pdpe);
                }
            }
        }
    }
}

fn init_idt() {
    let mut idtr: [IdtGateDescriptor; 256] = [Default::default(); 256];

    gdt::load(unsafe { &mut CPU_CONTEXT.gdt });
    seq!(N in 0..256 {
        let igtgd: IdtGateDescriptor = IdtGateDescriptor::new(
            crate::arch::internal::interrupts::isr::isr_handler~N as _,
            SegmentSelector {
                local_descriptor_table: false,
                index: 1, // This will cause issues lmao
                requested_privilege: CPL_RING_0,
            },
            IdtGateDescriptorProperties {
                gate_type: interrupts::idt::IdtGateType::Interrupt,
                privilege_level: CPL_RING_0,
            },
            0,
        );

        idtr[N] = igtgd;
    });

    unsafe {
        CPU_CONTEXT.idtr = Some(IdtDescriptor {
            size: (size_of::<IdtGateDescriptor>() * 256) as u16 - 1,
            idt_offset: (&idtr) as *const Idt,
        });
        interrupts::idt::load(CPU_CONTEXT.idtr.as_ref().unwrap());
        asm!("sti");
    }
}

#[allow(static_mut_refs)]
pub unsafe fn init() {
    gdt::load(unsafe { &mut CPU_CONTEXT.gdt });
    init_idt();

    unsafe {
        CPU_CONTEXT.info = Some(CpuInfo::new());
        CPU_CONTEXT
            .info
            .as_mut()
            .unwrap()
            .request(cpu::CpuIdRequest::BasicFeatures);
        info!(
            "APIC supported: {}",
            CPU_CONTEXT
                .info
                .as_ref()
                .unwrap()
                .basic_features
                .as_ref()
                .unwrap()
                .flags
                .contains(cpu::BasicFeaturesFlags::APIC)
        );
        // Unmask PIC
        asm!("mov al, 0x1", "out 0x21, al", "out 0xa1, al",);
        info!("PIC unmasked");

        info!("LD_TEXT_START: {:02x}", &raw const LD_TEXT_START as usize);
        info!("LD_TEXT_END: {:02x}", &raw const LD_TEXT_END as usize);
        info!(
            "LD_RODATA_START: {:02x}",
            &raw const LD_RODATA_START as usize
        );
        info!("LD_RODATA_END: {:02x}", &raw const LD_RODATA_END as usize);
        info!("LD_DATA_START: {:02x}", &raw const LD_DATA_START as usize);
        info!("LD_DATA_END: {:02x}", &raw const LD_DATA_END as usize);
    }
}

// Note: This is not in the generic section as each architecture has a dedicated linker script
unsafe extern "C" {
    pub unsafe static LD_TEXT_START: u8;
    pub unsafe static LD_TEXT_END: u8;
    pub unsafe static LD_RODATA_START: u8;
    pub unsafe static LD_RODATA_END: u8;
    pub unsafe static LD_DATA_START: u8;
    pub unsafe static LD_DATA_END: u8;
}
