//! Flashes the 8 onboard LEDs like a red alarm light.
//!
//! PIN GUESS: this targets PB0..PB7. It is still only a guess -- but note that
//! it *must* be a guess among pins that physically exist on this package. The
//! "C" in STM32F051**C**6 means LQFP48, which bonds out only:
//!
//!     PA0..PA15, PB0..PB15, PC13, PC14, PC15, PF0, PF1
//!
//! PC0..PC7 exist on the silicon (and the HAL happily hands them to you,
//! because the HAL is per-die, not per-package) but they are not connected to
//! any pin on a 48-pin part. Driving them compiles, flashes and runs, and does
//! nothing observable -- so don't use them as a guess.
//!
//! If nothing lights up, run the pin scanner:
//!
//!     cargo run --release --example scanner
//!
//! ...watch which pin(s) light an LED, then edit the pin list below to match.
#![no_main]
#![no_std]

use panic_halt as _;

use crate::hal::{delay::Delay, pac, prelude::*};
use stm32f0xx_hal as hal;

use cortex_m::peripheral::Peripherals as CorePeripherals;
use cortex_m_rt::entry;

/// Set to `true` if the LEDs are wired anode-to-3V3, cathode-to-pin (very
/// common on these kit IO boards), which makes them light on a LOW output.
const ACTIVE_LOW: bool = false;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

    let gpiob = p.GPIOB.split(&mut rcc);

    // Configure the 8 LED pins as push-pull outputs inside one critical
    // section (that's what into_push_pull_output() requires here), then erase
    // their types so they can live in one array.
    let mut leds = cortex_m::interrupt::free(|cs| {
        [
            gpiob.pb0.into_push_pull_output(cs).downgrade(),
            gpiob.pb1.into_push_pull_output(cs).downgrade(),
            gpiob.pb2.into_push_pull_output(cs).downgrade(),
            gpiob.pb3.into_push_pull_output(cs).downgrade(),
            gpiob.pb4.into_push_pull_output(cs).downgrade(),
            gpiob.pb5.into_push_pull_output(cs).downgrade(),
            gpiob.pb6.into_push_pull_output(cs).downgrade(),
            gpiob.pb7.into_push_pull_output(cs).downgrade(),
        ]
    });

    let mut delay = Delay::new(cp.SYST, &rcc);

    loop {
        for led in leds.iter_mut() {
            if ACTIVE_LOW {
                led.set_low().ok();
            } else {
                led.set_high().ok();
            }
        }
        delay.delay_ms(300_u16);

        for led in leds.iter_mut() {
            if ACTIVE_LOW {
                led.set_high().ok();
            } else {
                led.set_low().ok();
            }
        }
        delay.delay_ms(300_u16);
    }
}
