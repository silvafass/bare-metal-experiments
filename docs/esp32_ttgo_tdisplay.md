# ESP32 TTGO T-Display board

**Hardware Requirements**

- ESP32 TTGO T-Display board (or compatible)
- USB cable for programming

## Build from source in Ubuntu based systems

Follow the instructions in the link below to install the Rust development environment:
* https://rust-lang.org/tools/install/

Install espup and required Espressif Rust ecosystem:
```bash
cargo install espup --locked
espup install
# Set environment variables in bash shell
. $HOME/export-esp.sh
```
If you are running from nushell you can run below code to setting environment varaibles:
```bash
let esp_parsed_env = (
     open ($env.home)/export-esp.sh
     | str trim
     | lines
     | parse 'export {name}="{value}"'
 )
 $esp_parsed_env
     | where name != "PATH"
     | transpose --header-row --as-record
     | load-env
 let esp_path = (
     $esp_parsed_env
     | where name == "PATH"
     | get value
     | first
     | str replace ':$PATH' ''
 )
 $env.PATH = $env.PATH | prepend $esp_path
```
Install espflash:
```bash
cargo install espflash --locked
espflash board-info --port /dev/ttyUSB0 # get board information
```
* `espup` is a tool for installing and maintaining Espressif Rust ecosystem.
* `espflash` is a serial flasher utility for Espressif SoCs.

Clone repository locally
```bash
git clone git@github.com:silvafass/bare-metal-experiments.git
cd bare-metal-experiments
```

Build the binary:
```bash
cargo +esp build --target xtensa-esp32-none-elf --bin esp32-ttgo-tdisplay-blinks --release
```
And/or Flash the firmware by connecting the ESP32 TTGO T-Display board to the computer via USB and running the command below to update the board's firmware:
```bash
espflash flash --monitor --chip esp32 --port /dev/ttyUSB0 target/xtensa-esp32-none-elf/release/esp32-ttgo-tdisplay-blinks
```
Or just run it:
```bash
cargo +esp run --target xtensa-esp32-none-elf --bin esp32-ttgo-tdisplay-blinks --release
```

## Useful resources

* Official page for the [ESP32 microchip](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/get-started/index.html)
  * [ESP32 Datasheet](https://documentation.espressif.com/esp32_datasheet_en.pdf).
  * [ESP32 Techinical reference manual](https://documentation.espressif.com/esp32_technical_reference_manual_en.pdf)
* Some relevant references used during the development of this project.
  * [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)
  * [Embedded Rust (no_std) on Espressif](https://docs.espressif.com/projects/rust/no_std-training/)
