/* Memory layout of the ESP32 microcontroller */
/*  ->  MEMORY Command: */
/*      https://sourceware.org/binutils/docs/ld/MEMORY.html */
/*  ->  ESP32 chip memory types: */
/*      https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/memory-types.html */
MEMORY
{
  /* IRAM (Instruction RAM) / Internal SRAM 0 */
  /* Used to store parts of the application which need to run from RAM */
  IRAM (RWX) : ORIGIN = 0x40080000, LENGTH = 128K

  /* DRAM (Data RAM) / Internal SRAM 2 */
  /* Used for data (stack, heap, statics) */
  DRAM (RW)  : ORIGIN = 0x3FFAE000, LENGTH = 200K

  /* DROM (Data Stored in flash) */
  /* Used to be place constant data */
  DROM (R)   : ORIGIN = 0x3F400000, LENGTH = 2M

  /* IROM (Code Executed from flash) */
  /* Flash MMU is used to allow code execution from flash */
  IROM (RX)  : ORIGIN = 0x400D0000, LENGTH = 3M
}

/* Specify where the stack starts (usually end of DRAM) */
_stack_start = ORIGIN(DRAM) + LENGTH(DRAM);

/* The entry point ( reset handler ): The first instruction to execute */
ENTRY(reset_handler)

SECTIONS
{
  .rodata_desc :
  {
    *(.rodata_desc .rodata_desc.*)
  } > DROM

  .text :
  {
    *(.literal .literal.* .text .text.*)
  } > IROM

  .rodata :
  {
    *(.rodata .rodata.*);
  } > DROM

  .bss :
  {
    _sbss = .;
    *(.bss .bss.*)
    _ebss = .;
  } > DRAM

  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    _sdata = .;
    *(.data .data.*)
    _edata = .;
  } > DRAM

  _sidata = LOADADDR(.data);
}
