#![no_std]
#![no_main]
#![cfg(target_arch = "avr")]
#![feature(asm_experimental_arch)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

// CPU clock frequency in Hertz set to 16 MHz (16000000 cycles per second).
const F_CPU: u32 = 16_000_000;
// Each Assembly loop iteration takes 4 cpu cycles (2 for sbiw, 2 for brne)
const CPU_CYCLES_PER_LOOP: u32 = 4;
// How many Assembly loops are need to waste exactly 1 millisecond of clock
const LOOPS_PER_MS: u16 = (F_CPU / 1000 / CPU_CYCLES_PER_LOOP) as u16;

// # Register Definitions

// Raw pointers to specific memory address (in the Atmega328P, hardware pins are mapped to memory numbers).
// DDRB (Data Direction Register - Register Address 0x24): Controls Arduino Digital Pins 8 to 13.
const DDRB: *mut u8 = 0x24 as *mut u8;
// PORTB (Data Register - Register Address 0x25): Controls Arduino Digital Pins 8 to 13 with High (5V) or Low (0V)
const PORTB: *mut u8 = 0x25 as *mut u8;

// # Bit Settings Helpers

fn set_bit(reg_addr: *mut u8, bit_mask: u8) {
    unsafe { write_volatile(reg_addr, read_volatile(reg_addr) | bit_mask) }
}

fn clear_bit(reg_addr: *mut u8, bit_mask: u8) {
    unsafe { write_volatile(reg_addr, read_volatile(reg_addr) & !bit_mask) }
}

// # Dalay Logic implementation

// This function allow to pauses execution for a specific amount of time.
#[inline(never)]
fn delay_cycles(count: u16) {
    unsafe {
        let mut __count = count;
        // This is a loop in assembly that takes a total of 4 cycles per iteration
        // that uses the avr-libc implementation as a reference.
        // * See user manual here: https://www.nongnu.org/avr-libc/user-manual/group__util__delay__basic.html
        // * And the source code here: https://www.nongnu.org/avr-libc/user-manual/delay__basic_8h_source.html
        asm!(
            "1: sbiw {count}, 1", // Start of the loop and subtracts 1 from counter variable. this takes 2 CPU cycles
            "brne 1b", // If the result is not zero, jump back to label 1. This takes 2 CPU cycles
            count = inout(reg_pair) __count,
            options(nomem, nostack)
        )
    }
}

fn delay_ms(ms: u16) {
    for _ in 0..ms {
        // This waste 1 millisecond of CPU
        delay_cycles(LOOPS_PER_MS);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // DDRB Bit 5 = Pin 13
    const DDRB_BIT_05: u8 = 1 << 5;

    // Configure Port B (DDRB) as Output
    set_bit(DDRB, DDRB_BIT_05);

    loop {
        // Turn pin ON
        set_bit(PORTB, DDRB_BIT_05);
        delay_ms(1000);

        // Turn pin OFF
        clear_bit(PORTB, DDRB_BIT_05);
        delay_ms(1000);
    }
}
