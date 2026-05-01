# ESP32

## Installing Packages

```bash
# Espressif toolchain — installs xtensa target + LLVM fork
cargo install espup
espup install
. ~/export-esp.sh

# Optionally add to .bashrc / .zshrc
echo '. $HOME/export-esp.sh' >> ~/.bashrc

# Flash and monitor tool
cargo install espflash
cargo install cargo-espflash
cargo install esp-generate

# Some boards, like the S3 have built-in debug hardware
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Or via cargo
cargo install cargo-binstall
cargo binstall probe-rs-tools

# Can also do source install + compile but takes a long time to compile
# cargo install probe-rs-tools
```

## New project

Depends on the project, we may want to use `esp-hal` or `esp-idf`

|              | esp-generate (esp-hal)    | esp-idf-template             |
| ------------ | ------------------------- | ---------------------------- |
| Rust std     | ❌ no_std                 | ✅ full std                  |
| Underneath   | bare metal                | ESP-IDF (C + FreeRTOS)       |
| WiFi/BT      | esp-wifi (Rust, maturing) | mature, production-ready     |
| Compile time | fast                      | slow (builds all of ESP-IDF) |
| Binary size  | small                     | large                        |
| Complexity   | lower                     | higher setup                 |
| `println!`   | defmt / esp-println       | works natively               |
| ------------ | ------------------------- | ---------------------------- |

### esp-hal

For projects that are better suited to no-std

```bash
esp-generate --chip esp32s3 esp32s3_blink
cd esp32s3_blink
cargo run --release
```

On the _esp-generate_ menus:

- _Board_ ✅ `ESP32-S3-WROOM-2 (16/32MB flash, 8/16MB PSRAM)`
- _Optional editor integration_ ✅ Neovim

### esp-idf-hal

```bash
cargo install cargo-generate
cargo generate esp-rs/esp-idf-template
```

Follow the prompts — select:

- Target: **esp32s3**
- Enable PSRAM: **yes** (required for N16R8)
- ESP-IDF version: **v5.x** (latest stable)

### Flash

```bash
cargo espflash flash --monitor
```

Or as separate steps:

```bash
cargo build --release
cargo espflash flash --release
cargo espflash monitor
```

### Blink

`src/main.rs`:

```rust
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    loop {
        led.set_high()?;
        FreeRtos::delay_ms(500);
        led.set_low()?;
        FreeRtos::delay_ms(500);
    }
}
```

### Serial monitor

```bash
cargo espflash monitor
# or still use picocom
picocom -b 115200 /dev/<device>
```

### Neovim

rust-analyzer works out of the box via Mason — no extra LSP config needed
beyond your existing Rust setup:

```sh
:MasonInstall rust-analyzer
```

Ensure `rust-analyzer` is picking up the correct toolchain by confirming
`rust-toolchain.toml` exists in the project root (cargo-generate adds this
automatically):

```toml
[toolchain]
channel = "esp"
```
