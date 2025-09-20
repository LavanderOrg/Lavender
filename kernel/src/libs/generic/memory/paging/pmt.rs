use core::fmt::Display;

use crate::libs::arch::x86_64::memory::paging::{ADDRESS_MASK, PageEntryFlags};

pub struct PageMapTableEntry {
    inner: u64,
}

impl PageMapTableEntry {
    pub fn get_flags(&self) -> PageEntryFlags {
        PageEntryFlags::from_bits_truncate(self.inner)
    }

    pub fn get_address(&self) -> u64 {
        self.inner & ADDRESS_MASK
    }

    pub fn set_flags(&mut self, flags: PageEntryFlags) {
        self.inner = flags.bits();
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
            if flag.contains(PageEntryFlags::UserSupervisor) {
                "User"
            } else {
                "Supervisor"
            }
        )
    }
}
