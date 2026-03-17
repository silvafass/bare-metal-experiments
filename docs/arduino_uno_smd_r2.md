# Arduino UNO board

**Hardware Requirements**

- Arduino UNO board (or compatible)
- USB cable for programming

## Build from source in Ubuntu based systems

Install the required development dependencies:
```bash
sudo apt install gcc-avr avr-libc avrdude simavr
```
* `gcc-avr`and `avr-libc`are compillation/link and library dependencies for AVR microcontrollers (Atmel/Microchip).
* `avrdude` is a utility to program AVR microcontrollers.
* `simavr` is a lean AVR simulator.

Clone repository locally
```bash
git clone git@github.com:silvafass/bare-metal-experiments.git
cd bare-metal-experiments
```
Follow the instructions in the link below to install the Rust development environment:
https://rust-lang.org/tools/install/
And set the project to use the nightly version:
```bash
rustup toolchain install nightly
```

Build the binary:
```bash
cargo +nightly build --target avr-none --bin arduino-uno-smd-r2-blinks --release
```
So, you can quickly test it using the Simavr simulator:
```bash
simavr -m atmega328p ./target/avr-none/release/arduino-uno-smd-r2-blinks.elf
```
And/or Flash the firmware by connecting the Arduino Uno board to the computer via USB and running the command below to update the board's firmware:
```bash
avrdude -c arduino -P /dev/ttyACM0 -p atmega328p -D -U flash:w:target/avr-none/release/arduino-uno-smd-r2-blinks.elf:e
```

## Useful resources

* Official page for the [ATmega328p microchip](https://www.microchip.com/en-us/product/ATmega328P) and its respective [datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/MCU08/ProductDocuments/DataSheets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf).
* Official [pinout board documentation](https://docs.arduino.cc/resources/pinouts/A000073-full-pinout.pdf).
* Some relevant references used during the development of this project.
  * How to setting the [AVR target in Rust project](https://doc.rust-lang.org/rustc/platform-support/avr-none.html).
  * [Another bare minimal LED blinking example](https://github.com/dsvensson/rust-hello-atmega32u4) that was used as a essential reference.
