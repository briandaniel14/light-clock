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

## The board

This is the **UCT STM32F051C6 development board** (manufactured by Barracuda
Holdings, Western Cape). Identified from the silkscreen and the
`ADM1602K-NSA-FBS` 2-line LCD. Useful confirmed facts:

- The eight red LEDs **D0..D7 are PB0..PB7** — the silkscreen labels each one.
  `src/main.rs` is correct as written.
- The MCU sits on a **plug-in daughtercard**, not soldered to the main board.
  Its header runs in physical LQFP48 pin order, so SWD is at:

  ```
  ... PB4 PB3 PA15 PA14 PF7 PF6 PA13 PA12 ...
                   ^^^^         ^^^^
                   SWCLK        SWDIO
  ```

  Note PF7/PF6 sit *between* PA14 and PA13 — miscounting by two lands on an
  F-port pin and SWD silently fails.
- GND and 3V3 are on the P1 header along the top edge
  (`GND PA7 PA11 3V3 5V PA8`).
- Jumper defaults are printed on the silkscreen: JP1 `(1-2);(3-4)`,
  JP2 `ON`, P2 `(7-8);(9-10)`.

Related community resources: [STM32F0-Utilities](https://github.com/jonahswain/STM32F0-Utilities)
and [uct-stm32-dev-board-guides](https://github.com/ngakana/uct-stm32-dev-board-guides).

## If probe-rs can't connect (`JtagGetIdcodeError`)

```
WARN probe_rs::probe::stlink: send_jtag_command 242 failed: JtagGetIdcodeError
Error: Connecting to the chip was unsuccessful.
```

This is **not** a firmware problem. The ST-Link was found over USB, it drove
the SWD lines, and nothing answered with an IDCODE — so the debug port never
came up. Nothing in this repo has run on the chip yet at that point, so
rebuilding or changing the code cannot affect it. Work through these in
order:

1. **Wire 3V3 to the ST-Link, even if the board is powered elsewhere.** On a
   genuine ST-Link/V2 the 3.3V pin is a *reference* input for the level
   shifters (VAPP), not a supply. Leave it unconnected and the probe cannot
   drive SWDIO at all — this exact error. Four wires minimum:
   SWDIO, SWCLK, GND, 3V3.
2. **Check GND is actually common** and that SWDIO/SWCLK aren't swapped.
   Swapped is the second most common cause of this error.
3. **Confirm the probe itself is alive:**

   ```sh
   probe-rs list          # ST-Link should appear here
   probe-rs info          # talks raw SWD; prints the DP/AP IDs on success
   ```

   `probe-rs info` needs no `--chip` and no working firmware. If `list`
   shows the probe but `info` fails, the problem is on the four wires
   above, not in software.
4. **Try connecting under reset.** Needs NRST wired to the probe's RESET
   pin as a fifth wire. This holds the core in reset while attaching, which
   gets past a target that's in a low-power mode or stuck in lockup:

   ```sh
   probe-rs run --chip STM32F051C6Tx --connect-under-reset \
     target/thumbv6m-none-eabi/release/light-clock
   ```

   If that works, add `--connect-under-reset` to the `runner` line in
   `.cargo/config.toml` so plain `cargo run` uses it.
5. **Update ST-Link clone firmware.** Cheap V2 clones often ship with old
   firmware that probe-rs can't drive. ST's `STLinkUpgrade` utility
   reflashes them.
6. **Check BOOT0.** The daughtercard header has a `BOOT` pin; make sure it's
   tied low so the chip runs from flash.
7. **Reseat the MCU daughtercard.** The STM32 is on a removable module. If it
   isn't fully and squarely seated in its socket, SWD reaches nothing — and
   the board's power LED still lights, so it looks fine.

To find out whether the ST-Link is a separate dongle or built onto the board,
unplug everything and run `probe-rs list`, then plug in only the board's USB-B
and run it again. If a probe appears only on the second run, the programmer is
on the board and there is no SWD wiring for you to do.

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

## CI

`.github/workflows/ci.yml` runs on pushes to `main`, on every pull request,
and on demand from the Actions tab. Note that pushes to a *feature* branch
don't trigger it on their own — it's the open pull request that does, via
the `synchronize` event.

- **firmware** — builds `main.rs` and the scanner for `thumbv6m-none-eabi`,
  then runs `ci/check-firmware.sh`, which asserts the ELF is actually
  flashable: entry point set, `.vector_table` present at 0x08000000,
  non-empty `.text`, and the image fitting in 32K flash / 8K RAM.
- **lint** — `cargo fmt --check` and `cargo clippy -D warnings`.

The firmware check is the one worth having. `cargo build` exits 0 even when
the linker script isn't applied and the output contains no code at all —
that failure mode is what broke flashing here in the first place, and only
this step notices it. Run it yourself any time:

```sh
cargo build --release --bins --examples
./ci/check-firmware.sh \
  target/thumbv6m-none-eabi/release/light-clock \
  target/thumbv6m-none-eabi/release/examples/scanner
```

Note both cargo commands use `--bins --examples` rather than
`--all-targets`. This is a `no_std` crate, so `--all-targets` tries to build
a test harness that can't exist here and fails with ``can't find crate for
`test` ``.

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
