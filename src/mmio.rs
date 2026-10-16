#[inline]
pub unsafe fn read<T: Copy>(addr: usize) -> T {
    unsafe { (addr as *const T).read_volatile() }
}

#[inline]
pub unsafe fn write<T: Copy>(addr: usize, val: T) {
    unsafe { (addr as *mut T).write_volatile(val) }
}

#[inline]
pub unsafe fn set_bits(addr: usize, bits: u32) {
    unsafe { write(addr, read::<u32>(addr) | bits) }
}

#[inline]
pub unsafe fn clear_bits(addr: usize, bits: u32) {
    unsafe { write(addr, read::<u32>(addr) & !bits) }
}

// PROC_ID reads 0x55 on the CPU and 0xaa on the COP
pub const PROC_ID: usize = 0x60000000;
