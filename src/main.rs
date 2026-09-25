#![no_std]
#![no_main]

global_asm!(include_str!("asm/boot.s"));

const TEST_IMAGE: &[u8] = include_bytes!("../assets/test.data");

use core::arch::global_asm;
use crate::ipod_lcd::{lcd_backlight, lcd_init, lcd_invert, lcd_update, FRAMEBUFFER};

pub mod ipod_lcd;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
    }
}

#[unsafe(no_mangle)]
extern "C" fn _ipod_main() -> ! {
    lcd_init();
    lcd_backlight(true);

    // load test image
    for y in 0..110 {
        for x in 0..138 {
            let value = TEST_IMAGE[y * 138 + x];
            let bix = y * 138 * 4 + x * 2;
            let byx = bix / 8;
            let bix = ((bix % 8) / 2) * 2;
            let value = if value == 0xFF {
                0b1100_0000
            } else if value == 0x00 {
                0b00
            } else if value > 0x7F {
                0b1000_0000
            } else {
                0b0100_0000
            };

            unsafe {
                let val = (FRAMEBUFFER.0.get() as *mut u8).add(byx).read_volatile() & !(0b1100_0000 >> bix);
                (FRAMEBUFFER.0.get() as *mut u8).add(byx).write_volatile(val | (value >> bix));
            }
        }
    }

    lcd_update();
    
    loop {
    }
}