# sunlight-clock

Rust "flashing red lights" firmware for an STM32F051C6 board — a first step
towards the sunlight alarm clock project. Flashes the 8 onboard LEDs like a
red alarm light, plus a diagnostic tool for finding out exactly which pin
each LED (and button) is wired to, since this looks like an unbranded/kit
board with no datasheet to hand.

    +-o STM32 STLink@03100000  <class IOUSBHostDevice, id 0x10000287b, registered, matched, active, busy 0 (11 ms), retain 22>

## What this does

- `src/main.rs` — flashes 8 LEDs together, on 300ms / off 300ms. It's
  wired to guess pins **PB0–PB7**. That's a guess, but a guess constrained
  by the package: the "C" in STM32F051**C**6 means LQFP48, which only bonds
  out PA0–PA15, PB0–PB15, PC13–PC15 and PF0/PF1. PC0–PC7 exist on the die
  (the HAL will hand them to you) but reach no pin on a 48-pin part, so
  driving them silently does nothing.
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

`.cargo/config.toml` does three things, all of them required: it sets the
cross-compile target (`thumbv6m-none-eabi`), passes `-C link-arg=-Tlink.x`
so cortex-m-rt's linker script places the vector table at 0x08000000, and
sets the runner to `probe-rs run --chip STM32F051C6Tx` so `cargo run`
flashes. **Do not gitignore `.cargo/`** — without that file the build
succeeds but emits an ELF with no code in it at all, and there is nothing
to flash. Sanity-check any suspicious build with:

```sh
size target/thumbv6m-none-eabi/release/light-clock
```

`text` must be non-zero (~1KB for this firmware). If it reads 0, the linker
script isn't being applied.

If probe-rs doesn't recognize that exact chip name, run:

```sh
probe-rs chip list | grep -i stm32f051
```

and swap in whatever it prints, in `.cargo/config.toml`.

## If the LEDs don't light up on PB0-PB7

1. Look at the PCB silkscreen right next to the LEDs and the header pins —
   many of these boards print the pin name (e.g. "PA5") right by each one.
2. Run the scanner (`cargo run --release --example scanner`) and watch the
   board. Each sweep opens with a marker — every pin on for 1s, then all
   off for 1s — after which pins are driven one at a time, roughly two per
   second. Count from the marker; the numbered list at the top of
   `scanner.rs` maps each count to its pin.
3. If instead the whole board lights up and one LED goes *dark* as the
   sweep passes it, the LEDs are wired active-low. Set `ACTIVE_LOW = true`
   in both `src/main.rs` and `examples/scanner.rs` and reflash.
4. Ask your friend if a schematic or the board's product page/model number
   came with it — searching that model number usually turns up a pinout.
5. Worst case, a multimeter in continuity mode between an LED's pad and
   each header pin will find it directly.

Once you know the real pins, edit `src/main.rs` (swap e.g. `gpiob.pb0` for
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
