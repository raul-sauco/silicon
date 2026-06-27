//! ESP32-S3 Pin    RA-02 Pin    Wire
//! ───────────────────────────────────
//! 3.3V            3.3V         Red
//! GND             GND          Black
//! GPIO11          RST          Purple
//! GPIO6           NSS/CS       Blue
//! GPIO5           SCK          Yellow
//! GPIO10          MOSI         Green
//! GPIO7           MISO         White
//! GPIO9           DIO0         Orange

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    spi::{
        Mode,
        master::{Config as SpiConfig, Spi},
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use lora_phy::{
    LoRa,
    iv::GenericSx127xInterfaceVariant,
    mod_params::{Bandwidth, CodingRate, SpreadingFactor},
    sx127x::{Config as LoraConfig, Sx127x, Sx1276},
};
use panic_rtt_target as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const FREQ: u32 = 433_000_000;
const TX_POWER: i32 = 14;
const MSG: &[u8] = b"Hello S3!";

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3 -o esp32s3-wroom-2 -o probe-rs -o neovim -o embassy -o unstable-hal

    rtt_target::rtt_init_defmt!();

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

    info!("Embassy initialized!");

    // TODO: Spawn some tasks
    let _ = spawner;

    // LoRa setup
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(1))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO5)
    .with_mosi(peripherals.GPIO10)
    .with_miso(peripherals.GPIO7)
    .into_async();

    let cs = Output::new(peripherals.GPIO6, Level::High, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO11, Level::High, OutputConfig::default());
    let dio0 = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::None),
    );

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let iv = GenericSx127xInterfaceVariant::new(reset, dio0, None, None).unwrap();
    let sx = Sx127x::new(
        spi_dev,
        iv,
        LoraConfig {
            chip: Sx1276,
            tcxo_used: false,
            rx_boost: false,
            tx_boost: true,
        },
    );

    let mut lora = LoRa::new(sx, false, Delay).await.expect("LoRa handle");

    let mod_params = lora
        .create_modulation_params(
            SpreadingFactor::_7,
            Bandwidth::_125KHz,
            CodingRate::_4_5,
            FREQ,
        )
        .unwrap();

    let mut tx_params = lora
        .create_tx_packet_params(8, false, true, false, &mod_params)
        .unwrap();

    info!("LoRa TX ready - sending periodic Hello S3");

    loop {
        lora.prepare_for_tx(&mod_params, &mut tx_params, TX_POWER, MSG)
            .await
            .unwrap();

        match lora.tx().await {
            Ok(()) => info!("TX  → {=[u8]:a}", MSG),
            Err(_e) => defmt::warn!("TX error"),
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
