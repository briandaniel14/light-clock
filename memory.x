/* Linker memory map for the STM32F051C6 (Cortex-M0, 32K flash, 8K RAM). */
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 32K
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

