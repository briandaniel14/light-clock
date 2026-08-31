//! Pin scanner: finds out which pin each LED is actually wired to.
//!
//! Every sweep starts with a MARKER: all pins driven together for 1s, then all
//! off for 1s. After the marker, each pin is driven on its own for 400ms, then
//! all pins rest for 100ms, in exactly this order:
//!
//!      1 PA0     2 PA1     3 PA2     4 PA3     5 PA4     6 PA5
//!      7 PA6     8 PA7     9 PA8    10 PA9    11 PA10   12 PA11
//!     13 PA12   14 PA15   15 PB0    16 PB1    17 PB2    18 PB3
//!     19 PB4    20 PB5    21 PB6    22 PB7    23 PB8    24 PB9
//!     25 PB10   26 PB11   27 PB12   28 PB13   29 PB14   30 PB15
//!     31 PC13   32 PC14   33 PC15   34 PF0    35 PF1
//!
//! So: wait for the marker, then count 1-2-3... at roughly two per second. An
//! LED lighting on count 17 means it is on PB2.
//!
//! Two deliberate omissions:
//!   * PA13/PA14 are SWDIO/SWCLK. Driving them as plain GPIO drops the debug
//!     connection and can make the board hard to reflash until it is reset.
//!   * PC0..PC7 do not exist on the LQFP48 package this chip comes in (the "C"
//!     in STM32F051C6), so there is nothing to scan there.
//!
//! PF0/PF1 are OSC_IN/OSC_OUT. They are safe to drive when the chip runs off
//! the internal HSI (it does here), but if your board has a crystal fitted
//! they will not be LEDs -- expect nothing on counts 34/35.
#![no_main]
#![no_std]

use panic_halt as _;

use crate::hal::{
    delay::Delay,
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
};
use stm32f0xx_hal as hal;

use cortex_m::peripheral::Peripherals as CorePeripherals;
use cortex_m_rt::entry;

/// Set to `true` if the LEDs are wired anode-to-3V3, cathode-to-pin (very
/// common on these kit IO boards), which makes them light on a LOW output.
/// If the whole board lights up and one LED goes *dark* as the sweep passes
/// it, that is the tell -- flip this and re-flash.
const ACTIVE_LOW: bool = false;

fn drive(pin: &mut Pin<Output<PushPull>>, lit: bool) {
    if lit ^ ACTIVE_LOW {
        pin.set_high().ok();
    } else {
        pin.set_low().ok();
    }
}

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    // Keep this list in the same order as the table at the top of the file.
    let mut pins: [Pin<Output<PushPull>>; 35] = cortex_m::interrupt::free(|cs| {
        [
            gpioa.pa0.into_push_pull_output(cs).downgrade(),
            gpioa.pa1.into_push_pull_output(cs).downgrade(),
            gpioa.pa2.into_push_pull_output(cs).downgrade(),
            gpioa.pa3.into_push_pull_output(cs).downgrade(),
            gpioa.pa4.into_push_pull_output(cs).downgrade(),
            gpioa.pa5.into_push_pull_output(cs).downgrade(),
            gpioa.pa6.into_push_pull_output(cs).downgrade(),
            gpioa.pa7.into_push_pull_output(cs).downgrade(),
            gpioa.pa8.into_push_pull_output(cs).downgrade(),
            gpioa.pa9.into_push_pull_output(cs).downgrade(),
            gpioa.pa10.into_push_pull_output(cs).downgrade(),
            gpioa.pa11.into_push_pull_output(cs).downgrade(),
            gpioa.pa12.into_push_pull_output(cs).downgrade(),
            // PA13/PA14 skipped on purpose -- SWDIO/SWCLK.
            gpioa.pa15.into_push_pull_output(cs).downgrade(),
            gpiob.pb0.into_push_pull_output(cs).downgrade(),
            gpiob.pb1.into_push_pull_output(cs).downgrade(),
            gpiob.pb2.into_push_pull_output(cs).downgrade(),
            gpiob.pb3.into_push_pull_output(cs).downgrade(),
            gpiob.pb4.into_push_pull_output(cs).downgrade(),
            gpiob.pb5.into_push_pull_output(cs).downgrade(),
            gpiob.pb6.into_push_pull_output(cs).downgrade(),
            gpiob.pb7.into_push_pull_output(cs).downgrade(),
            gpiob.pb8.into_push_pull_output(cs).downgrade(),
            gpiob.pb9.into_push_pull_output(cs).downgrade(),
            gpiob.pb10.into_push_pull_output(cs).downgrade(),
            gpiob.pb11.into_push_pull_output(cs).downgrade(),
            gpiob.pb12.into_push_pull_output(cs).downgrade(),
            gpiob.pb13.into_push_pull_output(cs).downgrade(),
            gpiob.pb14.into_push_pull_output(cs).downgrade(),
            gpiob.pb15.into_push_pull_output(cs).downgrade(),
            gpioc.pc13.into_push_pull_output(cs).downgrade(),
            gpioc.pc14.into_push_pull_output(cs).downgrade(),
            gpioc.pc15.into_push_pull_output(cs).downgrade(),
            gpiof.pf0.into_push_pull_output(cs).downgrade(),
            gpiof.pf1.into_push_pull_output(cs).downgrade(),
        ]
    });

    let mut delay = Delay::new(cp.SYST, &rcc);

    loop {
        // Marker: everything on for 1s, everything off for 1s. This both
        // announces the start of a sweep and proves at least one LED is
        // reachable at all.
        for pin in pins.iter_mut() {
            drive(pin, true);
        }
        delay.delay_ms(1000_u16);
        for pin in pins.iter_mut() {
            drive(pin, false);
        }
        delay.delay_ms(1000_u16);

        // Sweep: one pin at a time, 400ms on / 100ms all-off.
        for pin in pins.iter_mut() {
            drive(pin, true);
            delay.delay_ms(400_u16);
            drive(pin, false);
            delay.delay_ms(100_u16);
        }
    }
}
