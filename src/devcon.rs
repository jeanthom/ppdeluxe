use crate::mmio;

pub const DEV_RS: usize = 0x60006004;
pub const DEV_RS2: usize = 0x60006008;
pub const DEV_EN: usize = 0x6000600c;
pub const DEV_EN2: usize = 0x60006010;

pub const CLCD_CLOCK_SRC: usize = 0x600060a0;

// bits in DEV_EN / DEV_RS
pub const DEV_SER0: u32 = 1 << 6;
pub const DEV_SER1: u32 = 1 << 7;

// bits in DEV_EN2 / DEV_RS2
pub const DEV2_CLCD: u32 = 1 << 13;

pub fn enable(bits: u32) {
    unsafe { mmio::set_bits(DEV_EN, bits) }
}

pub fn disable(bits: u32) {
    unsafe { mmio::clear_bits(DEV_EN, bits) }
}

pub fn enable2(bits: u32) {
    unsafe { mmio::set_bits(DEV_EN2, bits) }
}

// rockbox sleeps for a tick between assert and release, a short busy-wait does the job here
pub fn reset_pulse(bits: u32) {
    unsafe {
        mmio::set_bits(DEV_RS, bits);
        // volatile reads so the delay loop doesn't get optimised away
        for _ in 0..1000 {
            mmio::read::<u32>(DEV_RS);
        }
        mmio::clear_bits(DEV_RS, bits);
    }
}

pub fn clcd_clock_enable() {
    enable2(DEV2_CLCD);
}
