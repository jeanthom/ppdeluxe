#![no_std]
#![no_main]

global_asm!(include_str!("asm/boot.s"));

use core::arch::global_asm;
use crate::ipod_lcd::{lcd_backlight, lcd_init, lcd_invert, lcd_update};

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

    lcd_update();
    
    loop {
    }
}