/* # ESP32 Bare-Metal Linker Script
 * * The linker script tells the linker (ld) two main things:
 * 1. The physical/virtual memory layout of the target chip (the MEMORY block).
 * 2. Where to place different parts of the compiled program (code, variables,
 * constants) into that memory layout (the SECTIONS block).
 */

/* The MEMORY block defines the physical and virtual memory layout of the
 * ESP32 target. It tells the linker what memory regions are available,
 * their starting addresses (ORIGIN), their sizes (LENGTH), and their
 * access permissions (RWX - Read, Write, Execute).
 *
 * References
 * * MEMORY Command:
 * -> https://sourceware.org/binutils/docs/ld/MEMORY.html
 * * ESP32 chip memory types:
 * -> https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/memory-types.html
 */
MEMORY
{
  /* IRAM (Instruction RAM) / Internal SRAM 0
   * Permissions: Read, Write, Execute (RWX)
   * Usage: Extremely fast internal RAM. Used for critical code that must execute
   * quickly (like interrupt handlers) or code that needs to run while the flash
   * memory is being written/erased.
   */
  IRAM (RWX) : ORIGIN = 0x40080000, LENGTH = 128K

  /* DRAM (Data RAM) / Internal SRAM 2
   * Permissions: Read, Write (RW)
   * Usage: The main working memory for the application. It holds the stack,
   * the heap, and globally mutable data (like the .data and .bss sections).
   */
  DRAM (RW)  : ORIGIN = 0x3FFAE000, LENGTH = 200K

  /* DROM (Data Stored in flash)
   * Permissions: Read-only (R)
   * Usage: Read-only memory mapped directly from the external SPI Flash chip.
   * Used for constant variables, string literals, and the application descriptor.
   */
  DROM (R)   : ORIGIN = 0x3F400000, LENGTH = 2M

  /* IROM (Code Executed from flash)
   * Permissions: Read, Execute (RX)
   * Usage: Instruction memory mapped from the external SPI Flash chip.
   * The vast majority of the application's executable code (.text) lives here
   * and is fetched via the Flash MMU (Memory Management Unit) on demand.
   */
  IROM (RX)  : ORIGIN = 0x400D0000, LENGTH = 3M
}

/* Set the initial stack pointer address.
 * The stack typically grows downwards on this architecture. Placing it at
 * the very end of DRAM ensures it has maximum space to grow down toward
 * the heap and static variables.
 */
_stack_start = ORIGIN(DRAM) + LENGTH(DRAM);

/* ENTRY explicitly tells the linker which function is the absolute starting
 * point of the program. This matches the `reset_handler` function defined
 * in your Rust code.
 */
ENTRY(reset_handler)

/* The SECTIONS block dictates how the input sections from your compiled
 * Rust object files are mapped into the output memory regions defined above.
 */
SECTIONS
{
  /* ESP32 Application Descriptor
   * The bootloader expects a specific structure containing magic numbers
   * and version info to validate the firmware image. We explicitly place
   * the `.rodata_desc` section here so it lives in Flash (DROM).
   */
  .rodata_desc :
  {
    *(.rodata_desc .rodata_desc.*)
  } > DROM

  /* Executable Code Section
   * Mapped to IROM (Flash Executable memory).
   */
  .text :
  {
    /* NOTE: Xtensa architecture frequently places small constant pools
     * called "literals" right next to the instructions that use them.
     * We must include `.literal` sections alongside `.text` to prevent
     * linker errors or runtime crashes.
     */
    *(.literal .literal.* .text .text.*)
  } > IROM

  /* Read-Only Data Section
   * Mapped to DROM (Flash Data memory). Contains constant strings,
   * hardcoded arrays, and other immutable variables.
   */
  .rodata :
  {
    *(.rodata .rodata.*);
  } > DROM

  /* Block Started by Symbol (BSS) Section
   * Mapped to DRAM. This section is for global and static variables that
   * are uninitialized or explicitly initialized to zero in your code.
   * They don't take up space in the binary flash image.
   */
  .bss :
  {
    /* Save the current memory address into the `_sbss` symbol.
       Our Rust `reset_handler` uses this address to know where to start writing zeros. */
    _sbss = .;

    *(.bss .bss.*)

    /* Save the current memory address (after all .bss data) into the `_ebss` symbol.
       Our Rust `reset_handler` stops writing zeros when it reaches this address. */
    _ebss = .;
  } > DRAM

  /* * Initialized Data Section
   * Mapped to DRAM (VMA - Virtual Memory Address) so the CPU can read/write
   * it at runtime. However, the initial values must be stored in Flash.
   * The `AT(...)` command sets the LMA (Load Memory Address) to sit right
   * after the `.rodata` section in Flash.
   */
  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    /* Save the start address of the RAM location into `_sdata`. */
    _sdata = .;

    *(.data .data.*)

    /* Save the end address of the RAM location into `_edata`. */
    _edata = .;
  } > DRAM

  /* * Define a symbol for the Load Address of `.data` in Flash.
   * The startup code (`reset_handler`) will use `_sidata` as the source
   * address, and copy bytes to `_sdata` through `_edata` in RAM before
   * transferring control to `main()`.
   */
  _sidata = LOADADDR(.data);
}
