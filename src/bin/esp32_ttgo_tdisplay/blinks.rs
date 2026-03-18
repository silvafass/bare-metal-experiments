// # Bare-metal ESP32 backlight blink and UART0 for serial output example.

#![no_std]
#![no_main]
#![cfg(target_arch = "xtensa")]
#![feature(asm_experimental_arch)]

use core::arch::asm;
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr;

// ## GPIO Register Constants

// GPIO (0-31) enable write-1-to-set register.
// Setting a bit in this register configures the corresponding GPIO pin as an output.
const GPIO_ENABLE_W1TS_REG: *mut u32 = 0x3FF44024 as *mut u32;

// GPIO (0-31) output write-1-toset register.
// Writing a 1 to a bit here sets the corresponding GPIO output pin HIGH.
const GPIO_OUT_W1TS_REG: *mut u32 = 0x3FF44008 as *mut u32;

// GPIO (0-31) output write-1-to-clear register.
// Writing a 1 to a bit here sets the corresponding GPIO output pin LOW.
const GPIO_OUT_W1TC_REG: *mut u32 = 0x3FF4400C as *mut u32;

// ## RTC Register Constants

// RTC Watchdog Timer Write Protect register.
// Used to unlock the WDT registers before modifying them.
const RTC_CNTL_WDTWPROTECT_REG: *mut u32 = 0x3FF480A4 as *mut u32;

// RTC Watchdog Timer Configuration 0 register.
// Used to configure or disable the RTC Watchdog.
const RTC_CNTL_WDTCONFIG0_REG: *mut u32 = 0x3FF4808C as *mut u32;

// The main entry point for the application logic.
#[unsafe(no_mangle)]
fn main() -> ! {
    let mut uart = Uart0;

    // Define the bitmask for GPIO 4 (1 shifted left by 4)
    // GPIO pin 4 used here for display backlights on ESP32 TTGO T-Display board.
    const LED_GPIO: u32 = 1 << 4;

    unsafe {
        ptr::write_volatile(GPIO_ENABLE_W1TS_REG, LED_GPIO);
    }

    writeln!(uart, "\n=== Bare Metal ESP32 Booted! ===").ok();
    writeln!(uart, "Starting the display backlight blick sequence...").ok();

    loop {
        unsafe {
            // Set GPIO 4 HIGH
            ptr::write_volatile(GPIO_OUT_W1TS_REG, LED_GPIO);
        }
        writeln!(uart, "Display backlight: ON").ok();
        wait_cpu_cycles(15_000_000);

        unsafe {
            // Set GPIO 4 LOW
            ptr::write_volatile(GPIO_OUT_W1TC_REG, LED_GPIO);
        }
        writeln!(uart, "Display backlight: OFF").ok();
        wait_cpu_cycles(15_000_000);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// A zero-sized struct representing the primary UART peripheral (UART0)
struct Uart0;

impl core::fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // The memory address of the UART0 hardware FIFO buffer.
        const UART0_FIFO: *mut u8 = 0x3FF40000 as *mut u8;

        for &byte in s.as_bytes() {
            unsafe {
                // Write directly to the hardware FIFO byte by byte.
                ptr::write_volatile(UART0_FIFO, byte);
            }
        }

        Ok(())
    }
}

// A simple blocking delay function that wastes CPU cycles.
// This is not an accurate timer. Execution time waries based on CPU clock speed.
#[inline(never)]
fn wait_cpu_cycles(cpu_cycles: u32) {
    for _ in 0..cpu_cycles {
        unsafe {
            // "No Operation" assembly instruction forces the CPU to burn a cycle
            asm!("nop");
        }
    }
}

// The true entry point called by the ESP32 bootloader.
#[unsafe(no_mangle)]
pub extern "C" fn reset_handler() -> ! {
    // These symbols are defined in the linker script.
    // We use 'extern "C"' to tell Rust they are located elsewhere.
    unsafe extern "C" {
        static mut _sbss: u32; // Start of BSS (uninitialized data) in RAM
        static mut _ebss: u32; // End of BSS
        static mut _sdata: u32; // Start of DATA (initialized data) in RAM
        static mut _edata: u32; // End of DATA
        static _sidata: u32; // Start of initialization DATA in Flash (ROM)
    }

    unsafe {
        // Disable the RTC Watchdog Timer.
        // If enabled the ESP32 will reboot automatically after a
        // short time thinking the system has hung during initialization.
        ptr::write_volatile(RTC_CNTL_WDTWPROTECT_REG, 0x50D83AA1);
        ptr::write_volatile(RTC_CNTL_WDTCONFIG0_REG, 0);

        // Initialize .bss section to zero
        // This ensures that all uninitialized globals are actually 0.
        let mut sbss = &raw mut _sbss;
        let ebss = &raw mut _ebss;
        while sbss < ebss {
            ptr::write_volatile(sbss, 0);
            sbss = sbss.add(1);
        }

        // Copy .data section from Flash (VMA) to RAM (LMA)
        // This "brings to life" any global variables with initial values.
        let mut sdata = &raw mut _sdata;
        let edata = &raw mut _edata;
        let mut sidata = &raw const _sidata;
        while sdata < edata {
            ptr::write_volatile(sdata, ptr::read(sidata));
            sdata = sdata.add(1);
            sidata = sidata.add(1);
        }
    }

    // Tell the compiler that a safe Rust function named main exists.
    unsafe extern "Rust" {
        safe fn main() -> !;
    }

    // Transfer control to our main application code.
    main()
}

// Helper function to convert a standard Rust string into a C-style fixed-size byte array.
// Used to populate the ESP application descriptor.
const fn str_to_cstr<const C: usize>(s: &str) -> [u8; C] {
    let bytes = s.as_bytes();
    let mut data: [u8; C] = [0; C];
    let mut index = 0;
    loop {
        data[index] = bytes[index];
        index += 1;
        if index >= bytes.len() || index >= C {
            break;
        }
    }
    data
}

// Standard ESP-IDF App Descriptor Structure.
// The ESP32 bootloader looks for this struct in the binary to determine
// application metadata (magic word, version, project name).
#[repr(C)]
pub struct EspAppDesc {
    pub magic_word: u32,
    pub version: [u8; 32],
    pub project_name: [u8; 32],
}

// The global Application Descriptor instance.
// This is placed explicitly in the `.rodata_desc` section so the linker script
// can map it exactly where the ESP32 bootloader expects it.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata_desc")]
pub static APP_DESC: EspAppDesc = EspAppDesc {
    magic_word: 0xABCD5432, // Specific magic number required by the ESP bootloader
    version: str_to_cstr(env!("CARGO_PKG_VERSION")),
    project_name: str_to_cstr(env!("CARGO_PKG_NAME")),
};
