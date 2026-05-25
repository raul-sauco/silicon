//! ADXL345 accelerometer over I2C — ESP32-S3, esp-hal 1.1 no_std
//!
//! Wiring
//! ──────
//!   ADXL345   ESP32-S3
//!   VCC    →  3.3 V
//!   GND    →  GND
//!   SDA    →  GPIO 8
//!   SCL    →  GPIO 9
//!   SDO    →  GND    (I2C addr = 0x53; pull to 3.3V for 0x1D)
//!   CS     →  3.3 V  (selects I2C mode)

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use rtt_target::rprintln;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", panic_info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const ADXL345_ADDR: u8 = 0x53;

const REG_DEVID: u8 = 0x00;
const REG_POWER_CTL: u8 = 0x2D;
const REG_DATA_FORMAT: u8 = 0x31;
const REG_BW_RATE: u8 = 0x2C;
const REG_DATAX0: u8 = 0x32;

const SCALE_MG: f32 = 3.9;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
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

    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);

    let mut id = [0u8; 1];
    i2c.write_read(ADXL345_ADDR, &[REG_DEVID], &mut id).unwrap();
    if id[0] != 0xE5 {
        rprintln!("ADXL345 not found! Got 0x{:02X} (expected 0xE5)", id[0]);
        loop {}
    }
    rprintln!("ADXL345 found, device ID = 0x{:02X}", id[0]);

    i2c.write(ADXL345_ADDR, &[REG_BW_RATE, 0x0A]).unwrap();
    i2c.write(ADXL345_ADDR, &[REG_DATA_FORMAT, 0x08]).unwrap();
    i2c.write(ADXL345_ADDR, &[REG_POWER_CTL, 0x08]).unwrap();

    rprintln!("ADXL345 running at 100 Hz, ±2g full resolution");

    let mut buf = [0u8; 6];

    loop {
        i2c.write_read(ADXL345_ADDR, &[REG_DATAX0], &mut buf)
            .unwrap();

        let x = i16::from_le_bytes([buf[0], buf[1]]);
        let y = i16::from_le_bytes([buf[2], buf[3]]);
        let z = i16::from_le_bytes([buf[4], buf[5]]);

        let xf = x as f32 * SCALE_MG;
        let yf = y as f32 * SCALE_MG;
        let zf = z as f32 * SCALE_MG;

        // Raw force reads on mg
        // rprintln!("x={:.1}mg y={:.1}mg z={:.1}mg", xf, yf, zf);

        let pitch = libm::atan2f(xf, libm::sqrtf(yf * yf + zf * zf)).to_degrees();
        let roll = libm::atan2f(yf, libm::sqrtf(xf * xf + zf * zf)).to_degrees();
        rprintln!("pitch={:.1}° roll={:.1}°", pitch, roll);

        Timer::after(Duration::from_millis(10)).await;
    }
}
