//! Flashes the 8 onboard LEDs like a red alarm light.
//!
//! PIN GUESS: this targets PC0..PC7, because that's the single most common
//! wiring for these "STM32 core board + 8 LED / 5 button IO board" kits.
//! If nothing lights up, don't worry -- it means your board wires the LEDs
//! somewhere else. Run the pin scanner instead:
//!
//!     cargo run --example scanner
//!
//! ...watch which pin(s) light an LED, then edit the pin names below
//! (e.g. swap `gpioc.pc0` for `gpiob.pb0`) to match your board.
#![no_main]
#![no_std]

use panic_halt as _;

use crate::hal::{delay::Delay, pac, prelude::*};
use stm32f0xx_hal as hal;

use cortex_m::peripheral::Peripherals as CorePeripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

    let gpioc = p.GPIOC.split(&mut rcc);

    // Configure PC0..PC7 as push-pull outputs, all at once inside one
    // critical section (that's what into_push_pull_output() requires here).
    let (mut l0, mut l1, mut l2, mut l3, mut l4, mut l5, mut l6, mut l7) =
        cortex_m::interrupt::free(|cs| {
            (
                gpioc.pc7.into_push_pull_output(cs),
                gpioc.pc6.into_push_pull_output(cs),
                gpioc.pc5.into_push_pull_output(cs),
                gpioc.pc4.into_push_pull_output(cs),
                gpioc.pc3.into_push_pull_output(cs),
                gpioc.pc2.into_push_pull_output(cs),
                gpioc.pc1.into_push_pull_output(cs),
                gpioc.pc0.into_push_pull_output(cs),
            )
        });

    let mut delay = Delay::new(cp.SYST, &rcc);

    loop {
        l0.set_high().ok();
        l1.set_high().ok();
        l2.set_high().ok();
        l3.set_high().ok();
        l4.set_high().ok();
        l5.set_high().ok();
        l6.set_high().ok();
        l7.set_high().ok();
        delay.delay_ms(300_u16);

        l0.set_low().ok();
        l1.set_low().ok();
        l2.set_low().ok();
        l3.set_low().ok();
        l4.set_low().ok();
        l5.set_low().ok();
        l6.set_low().ok();
        l7.set_low().ok();
        delay.delay_ms(300_u16);
    }
}
