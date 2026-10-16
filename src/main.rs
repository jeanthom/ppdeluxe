#![no_std]
#![no_main]

extern crate alloc;

global_asm!(include_str!("asm/boot.s"));

use alloc::boxed::Box;
use alloc::string::String;
use core::arch::global_asm;
use core::fmt::Write;

use embassy_shell::{Args, Shell};

use crate::uart::AsyncConsole;

pub mod devcon;
pub mod heap;
pub mod mmio;
pub mod uart;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\npanic: {}", info);
    loop {
    }
}

#[derive(Clone, Copy)]
enum Width {
    U8,
    U16,
    U32,
}

impl Width {
    fn bytes(self) -> u32 {
        match self {
            Width::U8 => 1,
            Width::U16 => 2,
            Width::U32 => 4,
        }
    }

    fn max(self) -> u32 {
        match self {
            Width::U8 => 0xff,
            Width::U16 => 0xffff,
            Width::U32 => 0xffff_ffff,
        }
    }

    fn read(self, addr: u32) -> u32 {
        unsafe {
            match self {
                Width::U8 => mmio::read::<u8>(addr as usize) as u32,
                Width::U16 => mmio::read::<u16>(addr as usize) as u32,
                Width::U32 => mmio::read::<u32>(addr as usize),
            }
        }
    }

    fn write(self, addr: u32, val: u32) {
        unsafe {
            match self {
                Width::U8 => mmio::write::<u8>(addr as usize, val as u8),
                Width::U16 => mmio::write::<u16>(addr as usize, val as u16),
                Width::U32 => mmio::write::<u32>(addr as usize, val),
            }
        }
    }

    fn format(self, out: &mut String, val: u32) {
        let _ = match self {
            Width::U8 => write!(out, "{:02x}", val),
            Width::U16 => write!(out, "{:04x}", val),
            Width::U32 => write!(out, "{:08x}", val),
        };
    }
}

fn parse_hex(s: &str) -> Option<u32> {
    u32::from_str_radix(s, 16).ok()
}

fn parse_addr(s: Option<&str>, width: Width) -> Result<u32, &'static str> {
    let addr = s.and_then(parse_hex).ok_or("bad or missing address")?;
    if addr % width.bytes() != 0 {
        return Err("address isn't aligned to the access width");
    }
    Ok(addr)
}

fn cmd_read(width: Width, args: Args) -> Result<String, &'static str> {
    let addr = parse_addr(args.get(0), width)?;
    let count = match args.get(1) {
        Some(s) => s.parse().ok().filter(|&n: &u32| n > 0).ok_or("bad count")?,
        None => 1,
    };

    let mut out = String::new();
    for i in 0..count {
        let a = addr.wrapping_add(i * width.bytes());
        if i % 4 == 0 {
            if i != 0 {
                out.push_str("\r\n");
            }
            let _ = write!(out, "{:08x}:", a);
        }
        out.push(' ');
        width.format(&mut out, width.read(a));
    }
    Ok(out)
}

fn cmd_write(width: Width, args: Args) -> Result<String, &'static str> {
    let addr = parse_addr(args.get(0), width)?;
    let val = args.get(1).and_then(parse_hex).ok_or("bad or missing value")?;
    if val > width.max() {
        return Err("value doesn't fit in the access width");
    }

    width.write(addr, val);
    let mut out = String::new();
    let _ = write!(out, "{:08x} <- ", addr);
    width.format(&mut out, val);
    Ok(out)
}

async fn shell() -> ! {
    let mut shell = Shell::new();
    shell.prompt("pp> ");

    shell.add_command("r8", "r8 <addr> [count]: read count bytes from addr (hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_read(Width::U8, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });
    shell.add_command("r16", "r16 <addr> [count]: read count halfwords from addr (hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_read(Width::U16, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });
    shell.add_command("r32", "r32 <addr> [count]: read count words from addr (hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_read(Width::U32, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });
    shell.add_command("w8", "w8 <addr> <value>: write a byte to addr (both hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_write(Width::U8, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });
    shell.add_command("w16", "w16 <addr> <value>: write a halfword to addr (both hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_write(Width::U16, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });
    shell.add_command("w32", "w32 <addr> <value>: write a word to addr (both hex)", |args, mut io| {
        Box::pin(async move {
            match cmd_write(Width::U32, args) {
                Ok(out) => io.println(&out).await,
                Err(e) => io.println(e).await,
            }
        })
    });

    let _ = shell.run(&mut AsyncConsole, &mut AsyncConsole).await;

    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn _ipod_main() -> ! {
    let uart_result = uart::init();

    println!();
    println!("ppdeluxe on PP5020, PROC_ID=0x{:02x}", unsafe { mmio::read::<u32>(mmio::PROC_ID) } & 0xff);

    devcon::clcd_clock_enable();
    unsafe {
        println!("DEV_EN=0x{:08x} DEV_EN2=0x{:08x} CLCD_CLOCK_SRC=0x{:08x}",
                 mmio::read::<u32>(devcon::DEV_EN),
                 mmio::read::<u32>(devcon::DEV_EN2),
                 mmio::read::<u32>(devcon::CLCD_CLOCK_SRC));
    }

    match uart_result {
        Ok(()) => {
            heap::init();
            embassy_futures::block_on(shell())
        }
        _ => {},
    }

    loop {
    }
}
