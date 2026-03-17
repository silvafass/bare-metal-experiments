#![no_std]
#![no_main]
#![cfg(target_arch = "xtensa")]
#![feature(asm_experimental_arch)]

use core::arch::asm;
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr;

// --- GPIO Register Constants ---
const GPIO_ENABLE_W1TS_REG: *mut u32 = 0x3FF44024 as *mut u32;
const GPIO_OUT_W1TS_REG: *mut u32 = 0x3FF44008 as *mut u32;
const GPIO_OUT_W1TC_REG: *mut u32 = 0x3FF4400C as *mut u32;

// --- RTC Register Constants ---
const RTC_CNTL_WDTWPROTECT_REG: *mut u32 = 0x3FF480A4 as *mut u32;
const RTC_CNTL_WDTCONFIG0_REG: *mut u32 = 0x3FF4808C as *mut u32;

#[unsafe(no_mangle)]
fn main() -> ! {
    let mut uart = Uart0;

    const LED_GPIO: u32 = 1 << 4;

    unsafe {
        ptr::write_volatile(GPIO_ENABLE_W1TS_REG, LED_GPIO);
    }

    writeln!(uart, "\n=== Bare Metal ESP32 Booted! ===").ok();
    writeln!(uart, "Starting the display backlight blick sequence...").ok();

    loop {
        unsafe {
            ptr::write_volatile(GPIO_OUT_W1TS_REG, LED_GPIO);
        }
        writeln!(uart, "Display backlight: ON").ok();
        wait_cpu_cycles(15_000_000);

        unsafe {
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

struct Uart0;

impl core::fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        const UART0_FIFO: *mut u8 = 0x3FF40000 as *mut u8;

        for &byte in s.as_bytes() {
            unsafe {
                ptr::write_volatile(UART0_FIFO, byte);
            }
        }

        Ok(())
    }
}

fn wait_cpu_cycles(cpu_cycles: u32) {
    for _ in 0..cpu_cycles {
        unsafe {
            asm!("nop");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reset_handler() -> ! {
    unsafe extern "C" {
        static mut _sbss: u32;
        static mut _ebss: u32;
        static mut _sdata: u32;
        static mut _edata: u32;
        static _sidata: u32;
    }

    unsafe {
        ptr::write_volatile(RTC_CNTL_WDTWPROTECT_REG, 0x50D83AA1);
        ptr::write_volatile(RTC_CNTL_WDTCONFIG0_REG, 0);

        let mut sbss = &raw mut _sbss;
        let ebss = &raw mut _ebss;
        while sbss < ebss {
            ptr::write_volatile(sbss, 0);
            sbss = sbss.add(1);
        }

        let mut sdata = &raw mut _sdata;
        let edata = &raw mut _edata;
        let mut sidata = &raw const _sidata;
        while sdata < edata {
            ptr::write_volatile(sdata, ptr::read(sidata));
            sdata = sdata.add(1);
            sidata = sidata.add(1);
        }
    }

    unsafe extern "Rust" {
        safe fn main() -> !;
    }

    main()
}

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

#[repr(C)]
pub struct EspAppDesc {
    pub magic_word: u32,
    pub version: [u8; 32],
    pub project_name: [u8; 32],
}

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata_desc")]
pub static APP_DESC: EspAppDesc = EspAppDesc {
    magic_word: 0xABCD5432,
    version: str_to_cstr(env!("CARGO_PKG_VERSION")),
    project_name: str_to_cstr(env!("CARGO_PKG_NAME")),
};
