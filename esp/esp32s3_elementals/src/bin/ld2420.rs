#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config as UartConfig, Uart};
use heapless::String;
use rtt_target::rprintln;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", panic_info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// LD2420 protocol constants
const FRAME_HEADER: [u8; 4] = [0xFD, 0xFC, 0xFB, 0xFA];
const FRAME_FOOTER: [u8; 4] = [0x04, 0x03, 0x02, 0x01];

// Commands
const CMD_ENABLE_CONFIG: [u8; 2] = [0xFF, 0x00];
const CMD_DISABLE_CONFIG: [u8; 2] = [0xFE, 0x00];
const CMD_SET_TIMEOUT: [u8; 2] = [0x07, 0x00];

fn build_frame(command: &[u8], value: &[u8]) -> [u8; 32] {
    let mut frame = [0u8; 32];
    let data_len = (command.len() + value.len()) as u16;

    frame[0..4].copy_from_slice(&FRAME_HEADER);
    frame[4] = (data_len & 0xFF) as u8;
    frame[5] = (data_len >> 8) as u8;
    frame[6..6 + command.len()].copy_from_slice(command);
    frame[6 + command.len()..6 + command.len() + value.len()].copy_from_slice(value);
    let footer_start = 6 + command.len() + value.len();
    frame[footer_start..footer_start + 4].copy_from_slice(&FRAME_FOOTER);
    frame
}

#[derive(Debug, PartialEq, Eq)]
pub enum Presence {
    NotDetected,
    Detected { range_cm: u32 },
}

#[derive(Debug)]
pub struct SensorReport {
    pub presence: Presence,
}

impl SensorReport {
    /// Parses a block of text containing sensor readings.
    /// Handles both single-line "OFF" and multi-line "ON\nRange XYZ" patterns.
    pub fn parse(input: &str) -> Option<Self> {
        let mut lines = input.lines().map(|l| l.trim());

        while let Some(line) = lines.next() {
            if line == "OFF" {
                return Some(SensorReport {
                    presence: Presence::NotDetected,
                });
            } else if line == "ON" {
                // If it's ON, the next non-empty line should contain the range
                if let Some(range_line) = lines.find(|l| !l.is_empty()) {
                    if range_line.starts_with("Range ") {
                        if let Ok(range_cm) = range_line["Range ".len()..].parse::<u32>() {
                            return Some(SensorReport {
                                presence: Presence::Detected { range_cm },
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3 -o esp32s3-wroom-2 -o probe-rs -o neovim -o embassy -o unstable-hal

    rtt_target::rtt_init_print!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO0
    // - GPIO3
    // - GPIO45
    // - GPIO46
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO33;
    let _ = peripherals.GPIO34;
    let _ = peripherals.GPIO35;
    let _ = peripherals.GPIO36;
    let _ = peripherals.GPIO37;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    rprintln!("Embassy initialized!");

    // TODO: Spawn some tasks
    // let _ = spawner;

    // UART setup
    let uart_config = UartConfig::default().with_baudrate(115200);
    let mut uart: Uart<'_, Async> = Uart::new(peripherals.UART1, uart_config)
        .unwrap()
        .with_rx(peripherals.GPIO4) // UART wires cross, connect to sensor TX (OT1)
        .with_tx(peripherals.GPIO5) // Sensor RX
        .into_async();

    Timer::after_millis(100).await;

    rprintln!("Entering config mode...");
    let frame = build_frame(&CMD_ENABLE_CONFIG, &[0x00, 0x00]);
    uart.write_async(&frame[..14]).await.unwrap();
    uart.flush_async().await.unwrap();
    let mut resp = [0u8; 64];
    match uart.read_async(&mut resp).await {
        Ok(n) => rprintln!("RESP: {:02X?}", &resp[..n]),
        Err(e) => rprintln!("ERR: {:?}", e),
    }
    Timer::after_millis(100).await;

    rprintln!("Setting presence timeout to 5s");
    let frame = build_frame(&CMD_SET_TIMEOUT, &[0x05, 0x00]);
    uart.write_async(&frame[..14]).await.unwrap();
    Timer::after_millis(100).await;

    rprintln!("Exiting config mode");
    let frame = build_frame(&CMD_DISABLE_CONFIG, &[]);
    uart.write_async(&frame[..12]).await.unwrap();
    Timer::after_millis(100).await;

    rprintln!("Sensor configured: Reading presence data");

    // A heapless string buffer to accumulate incoming UART data
    // 128 bytes is plenty for a couple of lines of text
    let mut buffer = String::<128>::new();
    let mut rx_buf = [0u8; 32];
    let mut last_presence = Presence::NotDetected;

    // Some quick feedback LEDs
    let led_config = OutputConfig::default();
    let mut led_red = Output::new(peripherals.GPIO11, Level::Low, led_config);
    let mut led_yellow = Output::new(peripherals.GPIO12, Level::Low, led_config);
    let mut led_green = Output::new(peripherals.GPIO13, Level::Low, led_config);

    loop {
        match uart.read_async(&mut rx_buf).await {
            Ok(n) if n > 0 => {
                if let Ok(s) = core::str::from_utf8(&rx_buf[..n]) {
                    let _ = buffer.push_str(s);
                }

                if let Some(report) = SensorReport::parse(&buffer) {
                    // Only do something if the new reading is different from the last one
                    if report.presence != last_presence {
                        rprintln!("State changed to: {:?}", report.presence);
                        // If we have presence, check the range to light up the leds
                        match report.presence {
                            Presence::NotDetected => {
                                led_green.set_low();
                                led_yellow.set_low();
                                led_red.set_low();
                            }
                            Presence::Detected { range_cm } => {
                                if range_cm <= 50 {
                                    led_green.set_high();
                                    led_yellow.set_high();
                                    led_red.set_high();
                                } else if range_cm <= 120 {
                                    led_green.set_high();
                                    led_yellow.set_high();
                                    led_red.set_low();
                                } else {
                                    led_green.set_high();
                                    led_yellow.set_low();
                                    led_red.set_low();
                                }
                            }
                        }
                        last_presence = report.presence;
                    }

                    buffer.clear();
                }
            }
            Ok(_) => {}
            Err(e) => {
                rprintln!("UART read error: {:?}", e);
                buffer.clear();
            }
        }
    }
}
