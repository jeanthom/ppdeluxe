use core::cell::UnsafeCell;

pub const LCD1_CONTROL: usize = 0x70003000;
pub const LCD1_CMD: usize = 0x70003008;
pub const LCD1_BUSY_MASK: usize = 0x8000;

// pretty sure this is ipod mini 2g specific
pub const LCD_BACKLIGHT: usize = 0x6000d824;

// this is definitely model-specific
pub const LCD_WIDTH: usize = 138;
pub const LCD_HEIGHT: usize = 110;

pub struct FramebufferType(UnsafeCell<[u8; LCD_WIDTH * LCD_HEIGHT]>);

unsafe impl Sync for FramebufferType {}

pub static FRAMEBUFFER: FramebufferType = FramebufferType(UnsafeCell::new([0; LCD_WIDTH * LCD_HEIGHT]));

const R_START_OSC: usize = 0x00;
const R_DRV_OUTPUT_CONTROL: usize = 0x01;
const R_DRV_WAVEFORM_CONTROL: usize = 0x02;
const R_POWER_CONTROL: usize = 0x03;
const R_CONTRAST_CONTROL: usize = 0x04;
const R_ENTRY_MODE: usize = 0x05;
const R_ROTATION: usize = 0x06;
const R_DISPLAY_CONTROL: usize = 0x07;
const R_CURSOR_CONTROL: usize = 0x08;
const R_HORIZONTAL_CURSOR_POS: usize = 0x0b;
const R_VERTICAL_CURSOR_POS: usize = 0x0c;
const R_1ST_SCR_DRV_POS: usize = 0x0d;
const R_2ND_SCR_DRV_POS: usize = 0x0e;
const R_RAM_WRITE_MASK: usize = 0x10;
const R_RAM_ADDR_SET: usize = 0x11;
const R_RAM_DATA: usize = 0x12;

// specific to ipod mini 2g
const POWER_REG_H: usize = 0x1200;

const CONTRAST_REG_H: usize = 0x400;

// specific to ipod mini 2g
pub const DEFAULT_CONTRAST: usize = 42;

#[inline]
fn lcd_wait_write() {
    unsafe {
        while (LCD1_CONTROL as *mut usize).read_volatile() & LCD1_BUSY_MASK != 0 {}
    }
}

fn lcd_prepare_cmd(cmd: usize) {
    lcd_wait_write();
    unsafe {
        (LCD1_CMD as *mut usize).write_volatile(cmd | 0x740000);
    }
}

fn lcd_write_data(data: usize, width: usize) {
    for i in 0..width/2 {
        unsafe {
            let inputlo = (data as *mut u8).add(i*2).read_volatile();
            let inputhi = (data as *mut u8).add(i*2 +1).read_volatile();
            lcd_wait_write();
            (LCD1_CMD as *mut usize).write_volatile((inputhi as usize | ((inputlo as usize) << 8)) | 0x760000);
        }
    }
}

fn lcd_cmd_and_data(cmd: usize, data: usize) {
    // this is the ipod mini 2g implementation
    lcd_wait_write();
    unsafe {
        (LCD1_CMD as *mut usize).write_volatile(cmd | 0x740000);
    }
    lcd_wait_write();
    unsafe {
        (LCD1_CMD as *mut usize).write_volatile(data | 0x760000);
    }
}

// because i'm copying rockbox code, this is stored as 0..64
pub fn lcd_set_contrast(contrast: usize) {
    let contrast = contrast.clamp(0, 63);
    lcd_cmd_and_data(R_CONTRAST_CONTROL, CONTRAST_REG_H | (contrast + 64));
}

pub fn lcd_init() {
    for y in 0..LCD_HEIGHT {
        for x in 0..LCD_WIDTH {
            unsafe {
                (FRAMEBUFFER.0.get() as *mut u8).add(y * LCD_HEIGHT + x).write_volatile(0);
            }
        }
    }
    
    lcd_wait_write();
    unsafe {
        // specific to ipod mini 2g
        (LCD1_CONTROL as *mut usize).write_volatile(0x01730084); // constant from rockbox
    }

    lcd_cmd_and_data(R_DRV_WAVEFORM_CONTROL, 0x48);
    lcd_cmd_and_data(R_POWER_CONTROL, POWER_REG_H | 0xc);
    lcd_cmd_and_data(R_DISPLAY_CONTROL, 0x0019);
    lcd_set_contrast(DEFAULT_CONTRAST);
    lcd_cmd_and_data(R_ENTRY_MODE, 0x0000);
}

pub fn lcd_invert(new_invert: bool) {
    lcd_cmd_and_data(R_DISPLAY_CONTROL, if new_invert { 0x0027 } else { 0x0019 });
}

pub fn lcd_backlight(on: bool) {
    unsafe {
        (LCD_BACKLIGHT as *mut usize).write_volatile((0x100 + if on { 1 } else { 0 }) << 3);
    }
}

pub fn lcd_update_rect(fb: usize, mut x: usize, mut y: usize, mut width: usize, mut height: usize) {
    if x + width > LCD_WIDTH {
        width = LCD_WIDTH.saturating_sub(x);
    }
    if width == 0 {
        return;
    }
    let mut ymax = y + height.saturating_sub(1);
    ymax = ymax.min(LCD_HEIGHT - 1);

    for y in y..=ymax {
        lcd_cmd_and_data(R_RAM_ADDR_SET, (y << 5) + 20 - x);
        lcd_prepare_cmd(R_RAM_DATA);
        lcd_write_data(fb + (y * LCD_WIDTH) + x, width);
    }
}

pub fn lcd_update() {
    lcd_update_rect(FRAMEBUFFER.0.get() as usize, 0, 0, LCD_WIDTH, LCD_HEIGHT);
}