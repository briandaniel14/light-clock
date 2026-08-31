#!/usr/bin/env bash
#
# Validates that a built ELF is actually a flashable STM32F051C6 image.
#
# This exists because `cargo build` can exit 0 having produced a binary with
# no code in it whatsoever. If cortex-m-rt's linker script isn't applied
# (a missing `-C link-arg=-Tlink.x`), the linker finds no entry symbol,
# garbage-collects every allocated section, emits a warning most people
# scroll past, and succeeds. The result flashes "fine" and the chip does
# nothing. Every check below is cheap; that one is the reason for the file.
#
# Usage: ci/check-firmware.sh <elf> [<elf> ...]

set -euo pipefail

# STM32F051C6: 32 KiB flash, 8 KiB RAM. Must match memory.x.
readonly FLASH_BYTES=$((32 * 1024))
readonly RAM_BYTES=$((8 * 1024))
readonly FLASH_ORIGIN="08000000"

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <elf> [<elf> ...]" >&2
  exit 2
fi

fail() {
  echo "  FAIL: $*" >&2
  failed=1
}

overall=0

for elf in "$@"; do
  echo "== $(basename "$elf")"

  if [ ! -f "$elf" ]; then
    echo "  FAIL: no such file: $elf" >&2
    overall=1
    continue
  fi

  failed=0

  # --- entry point must be set ------------------------------------------
  # An unset entry point (0x0) is the signature of the linker script never
  # having run.
  entry=$(readelf -h "$elf" | awk -F: '/Entry point address/ { gsub(/ /,"",$2); print $2 }')
  if [ "$entry" = "0x0" ] || [ -z "$entry" ]; then
    fail "entry point is ${entry:-unset} -- linker script (-Tlink.x) was not applied"
  fi

  # --- vector table must exist, at the start of flash -------------------
  # The Cortex-M0 boots by reading the initial stack pointer and reset
  # vector from the very first bytes of flash. No table, no boot.
  # Strip the "[ N]" index column first: readelf pads it as "[ 1]" but "[10]",
  # which shifts awk's field numbering depending on section count.
  vt_addr=$(readelf -S -W "$elf" \
    | sed 's/^ *\[[ 0-9]*\] *//' \
    | awk '$1 == ".vector_table" { print $3 }')
  if [ -z "$vt_addr" ]; then
    fail "no .vector_table section -- the chip has no reset vector to boot from"
  elif [ "$vt_addr" != "$FLASH_ORIGIN" ]; then
    fail ".vector_table is at 0x$vt_addr, expected 0x$FLASH_ORIGIN"
  fi

  # --- there must be actual code ----------------------------------------
  read -r text data bss <<<"$(size "$elf" | awk 'NR==2 { print $1, $2, $3 }')"
  if [ -z "${text:-}" ]; then
    fail "could not read section sizes from $elf"
    text=0; data=0; bss=0
  fi

  if [ "$text" -eq 0 ]; then
    fail "text section is empty -- there is no code to flash"
  fi

  # --- must physically fit on the part ----------------------------------
  # Initialized data lives in flash too (it's copied into RAM at startup),
  # so it counts against both budgets.
  flash_used=$((text + data))
  ram_used=$((data + bss))

  if [ "$flash_used" -gt "$FLASH_BYTES" ]; then
    fail "flash: $flash_used B exceeds ${FLASH_BYTES} B"
  fi
  if [ "$ram_used" -gt "$RAM_BYTES" ]; then
    fail "RAM: $ram_used B exceeds ${RAM_BYTES} B"
  fi

  if [ "$failed" -eq 0 ]; then
    printf '  ok  entry %s, .vector_table @ 0x%s\n' "$entry" "$vt_addr"
    printf '  ok  flash %s/%s B (%s%%), RAM %s/%s B (%s%%)\n' \
      "$flash_used" "$FLASH_BYTES" "$((flash_used * 100 / FLASH_BYTES))" \
      "$ram_used" "$RAM_BYTES" "$((ram_used * 100 / RAM_BYTES))"
  else
    overall=1
  fi
done

exit "$overall"
