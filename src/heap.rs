// heap for embassy-shell, which needs alloc

use embedded_alloc::LlffHeap as Heap;

// in .bss, so it ends up in DRAM
const HEAP_SIZE: usize = 64 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

struct SingleCoreNoIrq;
critical_section::set_impl!(SingleCoreNoIrq);

unsafe impl critical_section::Impl for SingleCoreNoIrq {
    unsafe fn acquire() -> critical_section::RawRestoreState {}
    unsafe fn release(_: critical_section::RawRestoreState) {}
}

pub fn init() {
    unsafe {
        embedded_alloc::init!(HEAP, HEAP_SIZE);
    }
}
