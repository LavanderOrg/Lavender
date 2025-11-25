use seq_macro::seq;

use crate::libs::arch::x86_64::cpu::CpuInfo;
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
pub mod sse;
pub mod interrupts {
    pub mod ctx;
    pub mod idt;
    pub mod isr;
}

pub struct CpuContext {
    gdt: [u64; 5],
    idtr: Option<IdtDescriptor>,
    info: Option<CpuInfo>,
}

// NOTE: Yeah buddy you'll have to modify some of that for multi-proc support innit bruv
pub static mut CPU_CONTEXT: CpuContext = CpuContext {
    gdt: [0, 0, 0, 0, 0],
    idtr: None,
    info: None,
};

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

        sse::init().unwrap();

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
