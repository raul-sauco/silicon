# ESP32-S3 GC9A01 Hello World

A minimal Rust example displaying text on a GC9A01 1.28" round TFT LCD using an
ESP32-S3 DevKitC-1 N16R8.

## Hardware

- ESP32-S3 DevKitC-1 N16R8 (16MB flash, 8MB PSRAM)
- GC9A01 1.28" round TFT LCD (240x240, SPI)

## Wiring

| GC9A01 Pin | Label | ESP32-S3 Pin | Notes               |
| ---------- | ----- | ------------ | ------------------- |
| 1          | GND   | GND          |                     |
| 2          | VCC   | 3V3          |                     |
| 3          | SCL   | GPIO6        | SPI clock           |
| 4          | SDA   | GPIO7        | SPI MOSI            |
| 5          | RES   | GPIO5        | Reset               |
| 6          | DC    | GPIO4        | Data/Command select |
| 7          | CS    | GPIO10       | Chip select         |
| 8          | BLK   | 3V3          | Backlight — VCC     |

### Build and flash

```bash
cargo run --release
```

## Dependencies

| Crate               | Purpose                             |
| ------------------- | ----------------------------------- |
| `esp-hal`           | Hardware abstraction for ESP32-S3   |
| `mipidsi`           | Display driver for GC9A01           |
| `embedded-graphics` | Text and graphics primitives        |
| `embedded-hal-bus`  | SPI bus sharing (`ExclusiveDevice`) |
| `embedded-hal`      | Hardware abstraction traits         |

## Notes

- The GC9A01 is a 240×240 display. The physical screen is round but the
  framebuffer is square — pixels in the corners exist in memory but are hidden
  by the bezel.
- `SDA`/`SCL` labels on the display module look like I2C but are actually SPI.
- This project uses `no_std` bare metal Rust via `esp-hal`. No ESP-IDF, no FreeRTOS.
- Delay is implemented as a busy-wait using `esp_hal::time::Instant` — the
  `esp_hal::delay` module requires the `unstable` feature flag in esp-hal 1.1.0.
