# Embedded

Personal embedded systems playground. Starting with Arduino, likely moving
to ESP32 and beyond.

## Use

### Flash

```bash
arduino-cli compile --fqbn arduino:avr:uno <sketch>/
arduino-cli upload -p /dev/<device> --fqbn arduino:avr:uno <sketch>/
```

### Install libraries

On the dev machine local libraries are kept in `/lib` but not committed to git.

```bash
arduino-cli lib install --zip-path lib/<lib>.zip
```

### Serial

```bash
# picocom -b 9600 /dev/ttyACM0
picocom -b 9600 /dev/<device>
```

The -b 9600 is the baud rate — it must match what's in your sketch.
Exit using `Ctrl+A` then `Ctrl+X`.

## Setup

### Packages

```sh
sudo apt install clangd picocom

# Via package manager (or download binary from GitHub)
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh
# Move to PATH if needed
sudo mv bin/arduino-cli /usr/local/bin/

arduino-cli config init
arduino-cli core update-index
arduino-cli core install arduino:avr      # for Uno/Nano/Mega
# arduino-cli core install arduino:megaavr  # for Nano Every
# arduino-cli core install esp32:esp32      # for ESP32
```

### System

Write to device permissions.

**Permanent** Add your user to the dialout group. On Linux, serial ports like
/dev/ttyACM0 are owned by the dialout group, so only members can access them.
This is the permanent fix but requires a logout/login to take effect because
group memberships are loaded at session start.

```sh
usermod -aG dialout $USER
```

**Temporary** Directly gives read/write permission to everyone on that specific
device file right now, bypassing the group check. It works immediately but is
temporary — resets when you unplug and replug the board.

```sh
chmod a+rw /dev/<device>
```

### Neovim

```sh
:MasonInstall arduino-language-server
```

```lua
require('lspconfig').arduino_language_server.setup {
  cmd = {
    "arduino-language-server",
    "-clangd", "/usr/bin/clangd",   -- system clangd
    "-cli", "/usr/local/bin/arduino-cli",
    "-cli-config", vim.fn.expand("~/.arduino15/arduino-cli.yaml"),
    "-fqbn", "arduino:avr:uno",
  },
}
```

## ESP32-S3

### Required Packages

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Espressif toolchain — installs xtensa target + LLVM fork
cargo install espup
espup install

# Add to .bashrc / .zshrc
echo '. $HOME/export-esp.sh' >> ~/.bashrc

# Flash and monitor tool
cargo install cargo-espflash
```

#### New project

```bash
cargo install cargo-generate
cargo generate esp-rs/esp-idf-template
```

Follow the prompts — select:

- Target: **esp32s3**
- Enable PSRAM: **yes** (required for N16R8)
- ESP-IDF version: **v5.x** (latest stable)

#### Flash

```bash
cargo espflash flash --monitor
```

Or as separate steps:

```bash
cargo build --release
cargo espflash flash --release
cargo espflash monitor
```

#### Blink

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

#### Serial monitor

```bash
cargo espflash monitor
# or still use picocom
picocom -b 115200 /dev/<device>
```

#### Neovim

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
