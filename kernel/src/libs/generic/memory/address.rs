use core::{fmt::{Formatter, LowerHex}, ops::{Add, Sub}};
use core::ffi::c_void;
use crate::{
    debug, libs::{arch, generic::memory::paging::PaginationLevel}, KERNEL_CONTEXT
};

#[derive(Copy, Clone)]
pub struct VirtAddr(u64);

#[derive(Copy, Clone)]
pub struct PhysAddr(u64);

#[derive(Debug)]
pub struct NonCanonicalAddress();

// TODO: Move arch specific implementations to arch folder (ex: Map levels offsets)
impl VirtAddr {
    #[inline]
    pub fn get_level_offset(&self, level: PaginationLevel) -> u64 {
        if level == PaginationLevel::Physical {
            return self.0 & 0xFFF;
        }
        (self.0 >> 12 >> ((level as u64 - 1) * 9)) & 0x1FF
    }

    pub unsafe fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub unsafe fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub unsafe fn dump_offsets(&self) {
        debug!("{:02x} offsets: L5={:02x} L4={:02x} PDP={:02x} PD={:02x} PT={:02x}", self.0,
            self.get_level_offset(PaginationLevel::Level5),
            self.get_level_offset(PaginationLevel::Level4),
            self.get_level_offset(PaginationLevel::Level3),
            self.get_level_offset(PaginationLevel::Level2),
            self.get_level_offset(PaginationLevel::Level1),
        );
    }
}

impl Into<*mut u8> for VirtAddr {
    fn into(self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl Into<*mut c_void> for VirtAddr {
    fn into(self) -> *mut c_void {
        self.0 as *mut c_void
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

impl Into<u64> for VirtAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0 as usize
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

impl Into<u64> for PhysAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl LowerHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr { 0: self.0.wrapping_add(rhs as u64) }
    }
}

impl Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VirtAddr { 0: self.0.wrapping_add(rhs as u64) }
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr { 0: self.0.wrapping_sub(rhs.0) }
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
