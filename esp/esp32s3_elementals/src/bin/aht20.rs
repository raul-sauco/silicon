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
use esp_hal::{
    Blocking,
    clock::CpuClock,
    i2c::master::{Config as I2cConfig, I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
use rtt_target::rprintln;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", panic_info);
    loop {}
}

// ATH20 constants
const AHT20_ADDR: u8 = 0x38; // I2C address of AHT20
const CMD_INIT: u8 = 0xBE; // Initialize command
const INIT_PARAM1: u8 = 0x08;
const INIT_PARAM2: u8 = 0x00;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

    // Initialize Embassy runtime
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    rprintln!("Embassy initialized!");

    // We can use the spawner to read, display ... using different tasks
    // let _ = spawner;

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let mut i2c: I2c<'_, Blocking> = match I2c::new(peripherals.I2C0, i2c_config) {
        Ok(i2c) => i2c,
        Err(e) => {
            panic!("Failed to initialize I2C: {:?}", e)
        }
    }
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);

    // Let the AHT20 settle
    Timer::after_millis(200).await;

    // Initialize the AHT20
    match i2c.write(AHT20_ADDR, &[CMD_INIT, INIT_PARAM1, INIT_PARAM2]) {
        Ok(_) => {
            rprintln!("AHT20 initialized");
        }
        Err(e) => {
            rprintln!("Failed to initialize AHT20: {:?}", e);
        }
    }

    // AHT20 power-on cycle
    Timer::after_millis(100).await;

    let buf = &mut [0u8; 1];
    match i2c.read(AHT20_ADDR, buf) {
        Ok(_) => {
            if buf[0] & 0x08 == 0 {
                rprintln!("AHT20 is not calibrated, got status: 0x{:02X}", buf[0]);
                // Calibrate
                i2c.write(AHT20_ADDR, &[CMD_INIT, INIT_PARAM1, INIT_PARAM2])
                    .ok();
                Timer::after_millis(100).await;
            }
        }
        Err(e) => {
            rprintln!("Failed to read AHT20 status: {:?}", e);
        }
    }

    loop {
        // AHT20 constants for measurement
        const CMD_MEASURE: u8 = 0xAC;
        const MEASURE_PARAM1: u8 = 0x33;
        const MEASURE_PARAM2: u8 = 0x00;

        let mut aht20_data_valid = false;
        let mut humidity = 0.0f32;
        let mut aht20_temperature = 0.0f32;

        if i2c
            .write(AHT20_ADDR, &[CMD_MEASURE, MEASURE_PARAM1, MEASURE_PARAM2])
            .is_ok()
        {
            // Wait for measurement to complete (at least 80ms)
            Timer::after_millis(80).await;

            // Read 7 bytes of data
            let mut aht_buffer = [0u8; 7];
            if let Ok(_) = i2c.read(AHT20_ADDR, &mut aht_buffer) {
                // Check status bit for calibration
                if (aht_buffer[0] & 0x08) == 0 {
                    rprintln!("AHT20 sensor is not calibrated!");
                } else if (aht_buffer[0] & 0x80) != 0 {
                    rprintln!("AHT20 sensor is busy!");
                } else {
                    // Process humidity data (20 bits) from buffer[1], buffer[2], and buffer[3]
                    let humidity_raw = ((aht_buffer[1] as u32) << 12)
                        | ((aht_buffer[2] as u32) << 4)
                        | ((aht_buffer[3] as u32) >> 4);
                    humidity = (humidity_raw as f32) * 100.0 / 1048576.0;

                    // Process temperature data (20 bits) from buffer[3], buffer[4], and buffer[5]
                    let temp_raw = ((aht_buffer[3] as u32 & 0x0F) << 16)
                        | ((aht_buffer[4] as u32) << 8)
                        | (aht_buffer[5] as u32);
                    aht20_temperature = (temp_raw as f32) * 200.0 / 1048576.0 - 50.0;

                    aht20_data_valid = true;
                }
            } else {
                rprintln!("Failed to read data from AHT20");
            }
        } else {
            rprintln!("Failed to send measurement command to AHT20");
        }
        if aht20_data_valid {
            rprintln!(
                "AHT20: Temperature: {:.2} °C, Humidity: {:.2} %",
                aht20_temperature,
                humidity
            );
        } else {
            rprintln!("AHT20 data not valid");
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
