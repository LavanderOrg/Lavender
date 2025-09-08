use core::fmt::{Display, Formatter, LowerHex};

use crate::{
    KERNEL_CONTEXT,
    libs::{arch, generic::memory::paging::PaginationLevel},
};

#[derive(Copy, Clone)]
pub struct VirtAddr(u64);
pub struct PhysAddr(u64);
#[derive(Debug)]
pub struct NonCanonicalAddress(u64);

// TODO: Move arch specific implementations to arch folder (ex: Map levels offsets)
impl VirtAddr {
    #[inline]
    pub fn get_level_offset(&self, level: PaginationLevel) -> u64 {
        if level == PaginationLevel::Physical {
            return self.0 & 0xFFF;
        }
        (self.0 >> 12 >> ((level as u64 - 1) * 9)) & 0x1FF
    }
}

impl Into<*mut u8> for VirtAddr {
    fn into(self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl TryFrom<u64> for VirtAddr {
    type Error = NonCanonicalAddress;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if arch::paging::enforce_canonical() {
            // TODO: Check for canonical
        }
        Ok(VirtAddr { 0: value })
    }
}

impl LowerHex for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl PhysAddr {
    pub fn as_hhdm(&self) -> VirtAddr {
        unsafe {
            VirtAddr {
                0: self.0 | KERNEL_CONTEXT.boot_info.hhdm,
            }
        }
    }
}

impl From<u64> for PhysAddr {
    fn from(value: u64) -> Self {
        Self { 0: value }
    }
}

impl LowerHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::libs::generic::memory::{address::VirtAddr, paging::PaginationLevel};

    #[test]
    fn virtaddr_get_offsets() {
        let addr: VirtAddr = VirtAddr {
            0: 0x1BDAFE7EEFBE5CF,
        };

        assert_eq!(addr.get_level_offset(PaginationLevel::Physical), 0x5CF);
        assert_eq!(addr.get_level_offset(PaginationLevel::Level1), 0x1BE);
        assert_eq!(addr.get_level_offset(PaginationLevel::Level2), 0x177);
        assert_eq!(addr.get_level_offset(PaginationLevel::Level3), 0x19F);
        assert_eq!(addr.get_level_offset(PaginationLevel::Level4), 0x15F);
        assert_eq!(addr.get_level_offset(PaginationLevel::Level4), 0x1BD);
    }
}
