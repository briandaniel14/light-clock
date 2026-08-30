# sunlight-clock

Rust "flashing red lights" firmware for an STM32F051C6 board — a first step
towards the sunlight alarm clock project. Flashes the 8 onboard LEDs like a
red alarm light, plus a diagnostic tool for finding out exactly which pin
each LED (and button) is wired to, since this looks like an unbranded/kit
board with no datasheet to hand.

    +-o STM32 STLink@03100000  <class IOUSBHostDevice, id 0x10000287b, registered, matched, active, busy 0 (11 ms), retain 22>

## What this does

- `src/main.rs` — flashes 8 LEDs together, on 300ms / off 300ms. It's
  wired to guess pins **PC0–PC7**, because that's the single most common
  layout on these "STM32 core board + 8 LED / 5 button IO board" kits. It
  may just work first try.
- `examples/scanner.rs` — if the guess is wrong (nothing lights up, or the
  wrong things light up), this sweeps through nearly every GPIO pin one at
  a time, 400ms on / 100ms off, so you can watch the board and work out
  which pin each LED is actually on. The pin order is documented in a
  comment at the top of the file.

## One-time setup

```sh
rustup target add thumbv6m-none-eabi
cargo install probe-rs-tools --locked
```

`probe-rs-tools` installs `probe-rs` (what actually flashes the chip over
SWD) along with `cargo-flash`/`cargo-embed`. You'll also need an SWD
programmer wired to the board's SWDIO/SWCLK/GND/3V3 pins — a cheap ST-Link
V2 clone works fine — unless this board has one built onto it already
(check for a separate USB port near a "ST-Link" or "DAP" silkscreen label).

## Flashing

```sh
cargo run --release            # builds + flashes + runs src/main.rs
cargo run --release --example scanner   # builds + flashes + runs the pin scanner
```

`.cargo/config.toml` is set up to flash automatically via
`probe-rs run --chip STM32F051C6Tx` whenever you `cargo run`. If probe-rs
doesn't recognize that exact chip name, run:

```sh
probe-rs chip list | grep -i stm32f051
```

and swap in whatever it prints, in `.cargo/config.toml`.

## If the LEDs don't light up on PC0-PC7

1. Look at the PCB silkscreen right next to the LEDs and the header pins —
   many of these boards print the pin name (e.g. "PA5") right by each one.
2. Run the scanner (`cargo run --release --example scanner`) and watch the
   board. Whichever LED lights up, and when, tells you the real pin —
   count position in the sweep (or time it: roughly one pin every 0.5s)
   against the ordered list in the comment at the top of `scanner.rs`.
3. Ask your friend if a schematic or the board's product page/model number
   came with it — searching that model number usually turns up a pinout.
4. Worst case, a multimeter in continuity mode between an LED's pad and
   each header pin will find it directly.

Once you know the real pins, edit `src/main.rs` (swap e.g. `gpioc.pc0` for
whatever port/number you found) and reflash.

## Notes

- Targets the STM32F051C6 specifically: 32KB flash / 8KB RAM (`memory.x`),
  Cortex-M0 (`thumbv6m-none-eabi`), HAL feature `stm32f051`.
- Uses `stm32f0xx-hal` (synchronous, simplest for a quick blink). The main
  sunlight-clock project plan (in this Claude project) recommends
  `embassy-stm32` instead once you get to the async "wait until alarm time,
  then fade the light in" logic — this blinky is just to get something
  visibly working on the actual hardware first.
- PA13/PA14 (SWDIO/SWCLK) are deliberately left out of the scanner —
  driving those as plain GPIO can make the board hard to reflash until
  it's reset.
