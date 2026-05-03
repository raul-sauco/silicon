#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use esp_hal::time::{Duration, Instant};

use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::GC9A01;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_9X18_BOLD;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

esp_bootloader_esp_idf::esp_app_desc!();

struct BusyDelay;

impl embedded_hal::delay::DelayNs for BusyDelay {
    fn delay_ns(&mut self, ns: u32) {
        let end = Instant::now() + Duration::from_micros((ns as u64).div_ceil(1000));
        while Instant::now() < end {}
    }
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Control pins
    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    // SPI bus
    let spi_bus = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_3),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);

    // Wrap bus in ExclusiveDevice (handles CS)
    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, cs).unwrap();

    // mipidsi 0.10: SpiInterface needs a tx buffer
    let mut buffer = [0u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // Build display driver
    let mut display = Builder::new(GC9A01, di)
        .reset_pin(rst)
        .init(&mut BusyDelay)
        .unwrap();

    // Clear to black
    display.clear(Rgb565::BLACK).unwrap();

    // Draw text
    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
    Text::new("Hello Simei!", Point::new(60, 120), style)
        .draw(&mut display)
        .unwrap();

    loop {}
}
