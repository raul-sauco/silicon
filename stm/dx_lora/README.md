# LoRa TX/RX test project

Half-duplex UART↔LoRa bridge for the DX-Smart DX-LR30-900M22SP(T1) module,
written in Rust/Embassy. Type into the serial terminal, it transmits over
LoRa; anything received over LoRa gets printed back to the terminal, along
with RSSI/SNR.

## Hardware

- Board: DX-Smart DX-LR30-900M22SP(T1) — STM32F103C8T6 + SX1262 dev board
- MCU target: `stm32f103c8`
- No onboard debug probe (not a Nucleo/Discovery) — flashing goes through
  the STM32 UART bootloader, not `probe-rs`/SWD

### Pinout

| Signal | Pin  |
|--------|------|
| NSS    | PA4  |
| SCK    | PA5  |
| MOSI   | PA7  |
| MISO   | PA6  |
| RESET  | PA3  |
| BUSY   | PA2  |
| DIO1   | PC15 |
| RXEN   | PA1  |
| TXEN   | PA0  |
| Debug UART TX/RX | PA9 / PA10 |
| LED (undocumented, found by bisection) | PB11 |

Sourced from the vendor's Keil project (`UserConfig.h`) in the data package
linked below, cross-checked against the physical board.

## Frequency

Currently set to **915MHz** in `main.rs` (`FREQUENCY_HZ`) to match the
factory test firmware, for cross-testing this build against a board still
running stock firmware. **915MHz is outside the EU license-exempt band**
(EU/Spain uses 863–870MHz) — switch `FREQUENCY_HZ` to `868_000_000` before
any real-world / unattended use. TX power is capped at 14dBm in this
firmware, matching the EU 25mW limit (the factory firmware defaults to
22dBm, which does not).

## Prerequisites

- Rust with the `thumbv7m-none-eabi` target: `rustup target add thumbv7m-none-eabi`
- `cargo-binutils` + the `llvm-tools` component (for `cargo objcopy`)
- `stm32flash` (flashing over the UART bootloader)
- `picocom` or similar (serial terminal), 9600 baud

## Flashing this project

```sh
just flash
```

No debug probe is used here, so before running `just flash`, put the board
in bootloader mode:

1. Hold **Key/BOOT0**
2. Tap **Reset**, release it
3. Release **Key/BOOT0**

After flashing completes, **tap Reset again** — `stm32flash`'s `-g 0x0` jump
isn't equivalent to a real hardware reset, and the firmware may not behave
correctly until peripherals are reset from a clean state.

Other `just` targets: `just build` (compile only), `just serial` (open
`picocom` on the board), `just backup` (dump current flash, if read
protection isn't enabled).

## Restoring factory firmware

The stock DX-Smart test firmware (reads UART, transmits over LoRa at 915MHz;
listens and prints anything received) can be restored from the prebuilt hex
in the vendor's data package:

> `07 Programming code demonstration/LR20&30-900/LR20&30-900/Project/Objects/DX_TESET.hex`

Full package: [DX-LR30-900Mhz.Development.User.Information.rar][dx-rar]

```sh
stm32flash -w DX_TESET.hex -v -g 0x0 -b 115200 /dev/ttyUSB0
```

(same bootloader-mode dance as above)

## Usage

Connect via `picocom -b 9600 /dev/ttyUSB0`, type a line and press Enter to
transmit. Received packets print as:

```sh
RX >> <message>
RSSI: -42 dBm, SNR: 9 dB
```

## Known issues

- `lora.rx()` is currently raced inside `select()` against UART input, which
  the `lora-phy` crate's own docs warn against (`complete_rx` "is not safe to
  drop or cancel"). Works in testing so far, but should be reworked to use
  `RxMode::Single(n)` in a non-cancelled polling loop, with UART moved to
  its own task communicating via a channel, to avoid potential radio lockup.
- If a LoRa packet arrives mid-keystroke, the in-progress typed line is
  discarded (`select()` drops the `read_line` future).
- No onboard LED is documented by the vendor; PB11 was found by binary-search
  GPIO sweep and may not hold across board hardware revisions.

[dx-rar]: https://github.com/DX-SMART/LoRaModule/releases/download/DX-LR30-900Mhz/DX-LR30-900Mhz.Development.User.Information.rar
