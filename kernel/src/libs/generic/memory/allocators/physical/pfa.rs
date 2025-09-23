use crate::libs::generic::memory::address::PhysAddr;

pub trait PageFrameAllocator {
    fn allocate(clear: bool) -> PhysAddr;
    fn allocate_contiguous_range(size: usize, clear: bool) -> PhysAddr;
    fn free();
    fn available_total() -> usize;
    fn used() -> usize;
}
