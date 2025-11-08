use core::arch::asm;

use crate::{debug, libs::arch::x86_64::gdt::SegmentSelector};

#[repr(u8)]
pub enum IdtGateType {
    Interrupt = 0xE,
    Trap = 0xF,
}

pub struct IdtGateDescriptorProperties {
    pub gate_type: IdtGateType,
    pub privilege_level: u8,
}

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct IdtGateDescriptor {
    pub ep_ll: u16,
    pub segment_selector: u16,
    pub ist_offset: u8,
    pub properties: u8,
    pub ep_lh: u16,
    pub ep_hh: u32,
    _reserved: u32,
}

pub type Idt = [IdtGateDescriptor; 256];

#[repr(C, packed)]
pub struct IdtDescriptor {
    pub size: u16,
    pub idt_offset: *const Idt,
}

impl Into<u8> for IdtGateDescriptorProperties {
    fn into(self) -> u8 {
        let mut ret: u8 = 0;

        ret |= self.gate_type as u8;
        ret |= self.privilege_level << 5;
        ret |= 1 << 7;

        ret
    }
}

impl IdtGateDescriptor {
    pub fn new(
        entry_point: u64,
        segment_selector: SegmentSelector,
        properties: IdtGateDescriptorProperties,
        ist_offset: u8,
    ) -> Self {
        let mut ret = IdtGateDescriptor::default();

        ret.ep_ll = entry_point as u16;
        ret.segment_selector = segment_selector.into();
        ret.ist_offset = ist_offset & 0x7;
        ret.properties = properties.into();
        ret.ep_lh = (entry_point >> 16) as u16;
        ret.ep_hh = (entry_point >> 32) as u32;

        ret
    }
}

// TODO: Refactor
pub fn load(idtr: &IdtDescriptor) {
    debug!("Loading IDT at address {:02x}", idtr.idt_offset as u64);
    unsafe {
        asm!(
            "cli",
            "lidt [{idtr}]",
            idtr = in(reg) idtr
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::libs::arch::x86_64::{
        gdt::{CPL_RING_3, SegmentSelector},
        interrupts::idt::{
            IdtDescriptor, IdtGateDescriptor, IdtGateDescriptorProperties, IdtGateType,
        },
    };

    #[test]
    fn idt_test_serialize() {
        let segsel = SegmentSelector {
            local_descriptor_table: false,
            index: 0x546,
            requested_privilege: CPL_RING_3,
        };
        let properties = IdtGateDescriptorProperties {
            privilege_level: CPL_RING_3,
            gate_type: IdtGateType::Interrupt,
        };
        let idtgdesc = IdtGateDescriptor::new(0xF8D99A8B66647936, segsel, properties, 0b110);
        let ll = idtgdesc.ep_ll;
        let hh = idtgdesc.ep_hh;
        let lh = idtgdesc.ep_lh;

        assert_eq!(ll, 0x7936);
        assert_eq!(hh, 0xF8D99A8B);
        assert_eq!(lh, 0x6664);
        assert_eq!(size_of::<IdtGateDescriptor>() * 8, 128);
        assert_eq!(size_of::<IdtDescriptor>() * 8, 80);
        assert_eq!(size_of::<IdtGateDescriptor>() * 256, 4096);
    }
}
