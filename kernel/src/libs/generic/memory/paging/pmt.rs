use core::fmt::Display;

use crate::{debug, libs::{arch::x86_64::memory::paging::{PageEntryFlags, ADDRESS_MASK}, generic::memory::address::{PhysAddr, VirtAddr}}};

#[derive(Clone, Copy)]
pub struct PageMapTableEntry {
    inner: u64,
}

impl PageMapTableEntry {
    pub fn get_flags(&self) -> PageEntryFlags {
        PageEntryFlags::from_bits_truncate(self.inner)
    }

    pub fn get_address(&self) -> PhysAddr {
        //debug!("get_address(), inner {:02x}", self.inner);
        PhysAddr::from(self.inner & ADDRESS_MASK)
    }

    pub fn set_address(&mut self, addr: u64) {
        self.inner |= addr & ADDRESS_MASK;
        //debug!("set_address(0x{:02x}), inner {:02x}", self.inner & ADDRESS_MASK, self.inner);
    }

    pub fn set_flags(&mut self, flags: PageEntryFlags) {
        self.inner |= flags.bits();
    }
}

impl From<u64> for PageMapTableEntry {
    fn from(value: u64) -> Self {
        PageMapTableEntry { inner: value }
    }
}

impl Display for PageMapTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let flag = self.get_flags();

        write!(
            f,
            "0x{:02x} | {}, {}, {}",
            self.get_address(),
            if flag.contains(PageEntryFlags::Present) {
                "Present"
            } else {
                "Missing"
            },
            if flag.contains(PageEntryFlags::ReadWrite) {
                "Read/Write"
            } else {
                "Read-Only"
            },
            if flag.contains(PageEntryFlags::User) {
                "User"
            } else {
                "Supervisor"
            }
        )
    }
}
