// Inspired on
// https://github.com/Hahihula/esp32-c3-super-mini-rust/blob/master/examples/bmp280_aht20_board.rs
// worth having a look at:
// https://github.com/jorgeandrecastro/embassy-bmp280/blob/main/src/calibration.rs:
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use bme280_rs::{AsyncBme280, Configuration, Oversampling, SensorMode};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use esp_hal::Async;
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

    rprintln!("Embassy initialized!");

    // Give sensors time to power up before any I2C traffic
    Timer::after_millis(100).await;

    rprintln!("Scan complete");
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let mut i2c: I2c<'_, Async> = match I2c::new(peripherals.I2C0, i2c_config) {
        Ok(i2c) => i2c,
        Err(e) => {
            panic!("Failed to initialize I2C: {:?}", e);
        }
    }
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9)
    .into_async();

    // I2C scan
    rprintln!("Scanning I2C bus...");
    for addr in 1..=127u8 {
        let mut buf = [0u8];
        if i2c.read_async(addr, &mut buf).await.is_ok() {
            rprintln!("Device found at 0x{:02X}", addr);
        }
    }

    let mut bmp280 = AsyncBme280::new_with_address(i2c, 0x77, Delay);
    bmp280.init().await.expect("The BMP280 to initialize");
    bmp280
        .set_sampling_configuration(
            Configuration::default()
                .with_temperature_oversampling(Oversampling::Oversample1)
                .with_pressure_oversampling(Oversampling::Oversample1)
                .with_sensor_mode(SensorMode::Normal),
        )
        .await
        .expect("The BMP280 to be configured");
    loop {
        if let Some(temp) = bmp280.read_temperature().await.unwrap() {
            rprintln!("Temperature: {:.2} °C", temp);
        }
        if let Some(pressure) = bmp280.read_pressure().await.unwrap() {
            rprintln!("Pressure: {:.2} hPa", pressure / 100.0);
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
