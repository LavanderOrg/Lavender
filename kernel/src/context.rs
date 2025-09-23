use core::time::Duration;
use limine::{framebuffer::Framebuffer, paging::Mode, response::MemoryMapResponse};
use crate::libs::{drivers::logs::sinks::vga::Vga, generic::logging::logger::Logger};

#[derive(Default)]
pub struct BootInfo<'a> {
    pub limine_base_revision: Option<u64>,
    pub kernel_phys_address: u64,
    pub kernel_virt_address: u64,
    pub hhdm: u64,
    pub rtc_boot: Option<Duration>,
    pub paging_level: Option<Mode>,
    pub memory_map: Option<&'a MemoryMapResponse>,
}

#[derive(Default)]
pub struct KernelContext<'a> {
    pub framebuffer: Option<Framebuffer<'a>>,
    // I hate myself for this.
    pub vga: Option<Vga<'a>>,
    pub logger: Option<Logger<'a>>,
    pub boot_info: BootInfo<'a>,
}
