# ESP32

## References

- [ESP32-S3-WROOM2 Datasheet](https://documentation.espressif.com/esp32-s3-wroom-2_datasheet_en.pdf)
- [probe-rs Documentation](https://probe.rs/)
- [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)

### Electronics Books

#### Fundamentals

- [Practical Electronics for Inventors](https://www.mhprofessional.com/practical-electronics-for-inventors-fourth-edition-9781259587542-usa) — Scherz & Monk (4th ed.)
- [Learning the Art of Electronics](https://learningtheartofelectronics.com/) — Hayes & Horowitz (lab companion)
- [The Art of Electronics](https://artofelectronics.net/) — Horowitz & Hill (3rd ed.)
- [Make: Electronics](https://www.makershed.com/products/make-electronics-3rd-edition) — Charles Platt

#### Digital Electronics

- [Digital Design and Computer Architecture](https://www.elsevier.com/books/digital-design-and-computer-architecture-risc-v-edition/harris/978-0-12-820064-3) — Harris & Harris (RISC-V ed.)
- [The Art of Electronics: The x-Chapters](https://artofelectronics.net/the-x-chapters/) — Horowitz & Hill
- [Digital Electronics: A Practical Introduction](https://www.pearson.com/en-us/subject-catalog/p/digital-electronics-a-practical-approach-with-vhdl/P200000003258) — Kleitz

#### Embedded & Microcontrollers

- [The Embedded Rust Book](https://docs.rust-embedded.org/book/) — Rust Embedded Working Group
- [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) — Espressif
- [Programming Embedded Systems](https://www.oreilly.com/library/view/programming-embedded-systems/0596009836/) — Barr & Massa (O'Reilly)
- [Kolban's Book on ESP32](https://leanpub.com/kolban-ESP32) — Neil Kolban (free PDF)

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

## Tools

- [Wokwi][wokwi]

[wokwi]: https://wokwi.com/
