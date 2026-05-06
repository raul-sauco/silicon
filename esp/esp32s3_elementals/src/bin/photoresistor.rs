#![no_std]
#![no_main]
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    time::{Duration, Instant},
};

use rtt_target::{rprintln, rtt_init_print};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    rtt_init_print!();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(peripherals.GPIO3, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    // Manual timer for photoresistor reads
    let mut photoresistor_last_read = Instant::now();

    loop {
        if photoresistor_last_read.elapsed() >= Duration::from_millis(200) {
            photoresistor_last_read = Instant::now();
            let raw: u16 = nb::block!(adc.read_oneshot(&mut adc_pin)).unwrap();
            let voltage_mv = (raw as f32 / 4095.0) * 3300.0;
            rprintln!("raw={} voltage={}mV", raw, voltage_mv as u16);
        }
    }
}
