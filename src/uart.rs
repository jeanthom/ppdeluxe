// SER1 console on the dock connector. the 16550 itself is driven by the uart_16550 crate, this
// file only does the PP5020-specific bring-up. the sequence comes from the iPod mini 1G
// diagnostics image (internal ROM, loaded at 0x10000000, setup routine at 0x10007b4c), which
// uses SER1 as its console; it's the same as rockbox serial_setup() apart from the port and the
// pin routing. everything here is polled

use core::cell::UnsafeCell;
use core::fmt;
use core::ptr::NonNull;
use core::task::Poll;

use uart_16550::backend::MmioBackend;
use uart_16550::spec::registers::FifoTriggerLevel;
use uart_16550::{BaudRate, Config, InitError, Uart16550};

use crate::devcon;
use crate::mmio;

pub const SER1_BASE: usize = 0x70006040;
// registers are 32 bits apart
const SER1_STRIDE: u8 = 4;

const SER1_MCR: usize = SER1_BASE + 0x10;
const SER1_LSR: usize = SER1_BASE + 0x14;
// THRE | TEMT, the diag ROM waits for both
const LSR_TX_IDLE: u32 = 0x60;

// routes SER1 to the dock connector: bits 11:8 = 0b1010
const SER1_PIN_ROUTE: usize = 0x70000014;
const SER1_PIN_ROUTE_MASK: u32 = 0xf00;
const SER1_PIN_ROUTE_VAL: u32 = 0xa00;

// the UART is clocked at 24MHz and rockbox uses a divisor of 13 (24000000/13/16 = 115384 baud).
// uart_16550 refuses divisors that aren't an exact integer, so pretend the clock is exactly
// 16 * 13 * 115200 Hz: it then programs DLL=13, same as rockbox
const SER1_FAKE_FREQUENCY: u32 = 16 * 13 * 115_200;

pub struct ConsoleType(pub UnsafeCell<Option<Uart16550<MmioBackend>>>);

unsafe impl Sync for ConsoleType {}

pub static CONSOLE: ConsoleType = ConsoleType(UnsafeCell::new(None));

// pin routing, clock enable and reset
fn board_setup() {
    unsafe {
        let route = mmio::read::<u32>(SER1_PIN_ROUTE) & !SER1_PIN_ROUTE_MASK;
        mmio::write::<u32>(SER1_PIN_ROUTE, route | SER1_PIN_ROUTE_VAL);
    }
    devcon::disable(devcon::DEV_SER0);
    devcon::enable(devcon::DEV_SER1);
    devcon::reset_pulse(devcon::DEV_SER1);
}

pub fn init() -> Result<(), InitError> {
    board_setup();

    let mut uart = unsafe {
        Uart16550::new_mmio(NonNull::new(SER1_BASE as *mut u8).unwrap(), SER1_STRIDE)
            .expect("SER1 base address and stride are valid")
    };
    uart.init(Config {
        frequency: SER1_FAKE_FREQUENCY,
        baud_rate: BaudRate::Baud115200,
        // FCR=0x07 like in diagnostics
        fifo_trigger_level: Some(FifoTriggerLevel::One),
        ..Config::DEFAULT
    })?;
    unsafe {
        mmio::write::<u32>(SER1_MCR, 0x03);
    }

    unsafe {
        *CONSOLE.0.get() = Some(uart);
    }
    Ok(())
}

pub fn uart() -> Option<&'static mut Uart16550<MmioBackend>> {
    unsafe { (*CONSOLE.0.get()).as_mut() }
}

pub fn putc(c: u8) {
    if let Some(uart) = uart() {
        uart.send_bytes_exact(&[c]);
    }
}

pub struct AsyncConsole;

impl embedded_io_async::ErrorType for AsyncConsole {
    type Error = core::convert::Infallible;
}

impl embedded_io_async::Read for AsyncConsole {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        Ok(core::future::poll_fn(|cx| {
            let n = uart().map_or(0, |uart| uart.receive_bytes(buf));
            if n > 0 {
                Poll::Ready(n)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await)
    }
}

impl embedded_io_async::Write for AsyncConsole {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if let Some(uart) = uart() {
            uart.send_bytes_exact(buf);
        }
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        unsafe {
            while mmio::read::<u32>(SER1_LSR) & LSR_TX_IDLE != LSR_TX_IDLE {}
        }
        Ok(())
    }
}

pub struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            if c == b'\n' {
                putc(b'\r');
            }
            putc(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::uart::Console, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($crate::uart::Console, $($arg)*);
    }};
}
