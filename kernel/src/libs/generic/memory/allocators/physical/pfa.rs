use crate::libs::generic::memory::address::PhysAddr;

pub trait PageFrameAllocator {
    fn allocate(&mut self, clear: bool) -> PhysAddr;
    fn free(&mut self);
    fn available_total(&self) -> usize;
    fn used(&self) -> usize;
}
