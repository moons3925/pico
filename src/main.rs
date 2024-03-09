//! # UART の受信割り込みを使ったサンプル
//!
//! ピンアサイン:
//!
//! * GPIO 0 - UART TX (out of the RP2040)
//! * GPIO 1 - UART RX (in to the RP2040)
//! * GPIO 25 - An LED we can blink (active high)
//!

#![no_std] // (8-1-1)
#![no_main] // (8-1-2)

use core::cell::RefCell;
use core::ops::DerefMut; // (8-2-1)

use rp2040_hal::Clock; // (8-2-2)

use rp_pico::entry; // (8-2-3)

use fugit::RateExtU32; // (8-2-4)

use hal::pac; // (8-2-5)

use hal::pac::interrupt; // (8-2-6)

use critical_section::Mutex; // (8-2-7)

use hal::gpio::bank0::{Gpio0, Gpio1}; // (8-2-8)

use hal::uart::{DataBits, StopBits, UartConfig}; // (8-2-9)

use embedded_hal::{
    // (8-2-10)
    digital::v2::OutputPin,
    serial::{Read, Write},
};

use rp2040_hal as hal; // (8-2-11)

use panic_halt as _; // (8-2-12)

type UartPins = (
    // (8-3-1)
    hal::gpio::Pin<Gpio0, hal::gpio::FunctionUart, hal::gpio::PullNone>,
    hal::gpio::Pin<Gpio1, hal::gpio::FunctionUart, hal::gpio::PullNone>,
);

type Uart = hal::uart::UartPeripheral<hal::uart::Enabled, pac::UART0, UartPins>; // (8-3-2)

static GLOBAL_UART: Mutex<RefCell<Option<Uart>>> = Mutex::new(RefCell::new(None)); // (8-4-1)

#[entry] // (8-1-3)
fn main() -> ! {
    // (8-5-1)
    let mut pac = pac::Peripherals::take().unwrap(); // (8-6-1)
    let core = pac::CorePeripherals::take().unwrap(); // (8-6-2)

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG); // (8-6-3)

    let clocks = hal::clocks::init_clocks_and_plls(
        // (8-6-4)
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz()); // (8-6-5)

    let sio = hal::Sio::new(pac.SIO); // (8-6-6)

    let pins = rp_pico::Pins::new(
        // (8-6-7)
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        // (8-6-8)
        pins.gpio0.reconfigure(),
        pins.gpio1.reconfigure(),
    );

    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS) // (8-6-9)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    unsafe {
        // (8-6-10)
        pac::NVIC::unmask(hal::pac::Interrupt::UART0_IRQ);
    }

    uart.enable_rx_interrupt(); // (8-6-11)

    uart.write_full_blocking(b"uart_interrupt example started...\n"); // (8-6-12)

    critical_section::with(|cs| {
        // (8-6-13)
        GLOBAL_UART.borrow(cs).replace(Some(uart));
    });

    // But we can blink an LED.
    let mut led_pin = pins.led.into_push_pull_output(); // (8-6-14)

    loop {
        // (8-6-15)
        cortex_m::asm::wfe(); // (8-6-16)
        led_pin.set_high().unwrap(); // (8-6-17)
        delay.delay_ms(100); // (8-6-18)
        led_pin.set_low().unwrap(); // (8-6-19)
    }
}

#[interrupt] // (8-1-4)
fn UART0_IRQ() {
    // (8-6-20)
    critical_section::with(|cs| {
        // (8-6-21)
        if let Some(ref mut uart) = GLOBAL_UART.borrow(cs).borrow_mut().deref_mut() {
            // (8-6-22)
            while let Ok(byte) = uart.read() {
                // (8-6-23)
                let _ = uart.write(byte + 1); // (8-6-24)
            }
        }
    });
    cortex_m::asm::sev(); // (8-6-25)
}
