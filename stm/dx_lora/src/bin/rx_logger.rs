// Signal      Port/Pin
//
// NSS         PA4
// SCK         PA5
// MOSI        PA7
// MISO        PA6
// RESET       PA3
// BUSY        PA2
// DIO1        PC15
// UART_TX     PA9     debug
// UART_RX     PA10    debug
// RXEN        PA1
// TXEN        PA0
// Debug UART TX/RX	PA9 / PA10

#![no_std]
#![no_main]

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use core::fmt::Write;
use dx_lora::{FREQUENCY_HZ, WaterMeterPayload};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    interrupt, peripherals,
    spi::{Config as SpiConfig, Spi},
    time::Hertz,
    usart::{Config as UartConfig, Uart},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Delay, Duration, Timer};
use lora_phy::{
    LoRa, RxMode,
    iv::GenericSx126xInterfaceVariant,
    mod_params::{Bandwidth, CodingRate, SpreadingFactor},
    sx126x::{Sx126x, Sx1262},
};

bind_interrupts!(struct Irqs {
    USART1 => embassy_stm32::usart::InterruptHandler<peripherals::USART1>;
    DMA1_CHANNEL2 => embassy_stm32::dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_CHANNEL3 => embassy_stm32::dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_CHANNEL4 => embassy_stm32::dma::InterruptHandler<peripherals::DMA1_CH4>;
    DMA1_CHANNEL5 => embassy_stm32::dma::InterruptHandler<peripherals::DMA1_CH5>;
    EXTI2 => embassy_stm32::exti::InterruptHandler<interrupt::typelevel::EXTI2>;
    EXTI15_10 => embassy_stm32::exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

const BAUD_RATE: u32 = 9600;

static LED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut uart_config = UartConfig::default();
    uart_config.baudrate = BAUD_RATE;
    let mut uart = Uart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Irqs,
        uart_config,
    )
    .expect("UART struct");

    uart.write(b"\r\n").await.ok();
    uart.write(b"##############################\r\n").await.ok();
    uart.write(b"LoRa TX/RX half-duplex example\r\n").await.ok();
    uart.write(concat!("Firmware v", env!("CARGO_PKG_VERSION"), "\r\n").as_bytes())
        .await
        .ok();
    uart.write(b"##############################\r\n").await.ok();

    // SPI1 to SX1262
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi = Spi::new(
        p.SPI1, p.PA5, /*SCK*/
        p.PA7, /*MOSI*/
        p.PA6, /*MISO*/
        p.DMA1_CH3, p.DMA1_CH2, Irqs, spi_config,
    );

    let nss = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi_dev = embedded_hal_bus::spi::ExclusiveDevice::new(spi, nss, Delay).unwrap();

    let reset = Output::new(p.PA3, Level::High, Speed::VeryHigh);
    let busy = ExtiInput::new(p.PA2, p.EXTI2, Pull::None, Irqs);
    let dio1 = ExtiInput::new(p.PC15, p.EXTI15, Pull::None, Irqs);
    let rxen = Output::new(p.PA1, Level::Low, Speed::VeryHigh); // external RF-switch pins,
    let txen = Output::new(p.PA0, Level::Low, Speed::VeryHigh); // not part of the SX1262 itself

    let iv = GenericSx126xInterfaceVariant::new(reset, dio1, busy, Some(rxen), Some(txen)).unwrap();

    let config = lora_phy::sx126x::Config {
        chip: Sx1262,
        tcxo_ctrl: None, // this module doesn't appear to have a TCXO — set only if yours does
        use_dcdc: true,
        rx_boost: false,
    };
    let mut lora = LoRa::new(Sx126x::new(spi_dev, iv, config), false, Delay)
        .await
        .unwrap();

    let mdltn_params = lora
        .create_modulation_params(
            SpreadingFactor::_9,
            Bandwidth::_125KHz,
            CodingRate::_4_6,
            FREQUENCY_HZ,
        )
        .unwrap();

    uart.write(b"lora rx_logger listening\r\n").await.ok();

    // LoRa RX
    let rx_pkt_params = lora
        .create_rx_packet_params(8, false, 64, true, false, &mdltn_params)
        .unwrap();
    lora.prepare_for_rx(RxMode::Continuous, &mdltn_params, &rx_pkt_params)
        .await
        .unwrap();
    let mut rx_buf = [0u8; 64];

    let led = Output::new(p.PB11, Level::High, Speed::Low);
    spawner.spawn(led_task(led)).unwrap();

    loop {
        lora.prepare_for_rx(RxMode::Continuous, &mdltn_params, &rx_pkt_params)
            .await
            .unwrap();
        if let Ok((len, status)) = lora.rx(&rx_pkt_params, &mut rx_buf).await {
            let mut fmt_buf: heapless::String<96> = heapless::String::new();

            match WaterMeterPayload::from_bytes(&rx_buf[..len as usize]) {
                Some(p) => {
                    let _ = write!(
                        fmt_buf,
                        "{},{},{}\r\n",
                        p.to_log_string(),
                        status.rssi,
                        status.snr
                    );
                }
                None => {
                    let _ = write!(
                        fmt_buf,
                        "bad packet len={} rssi={} snr={}\r\n",
                        core::str::from_utf8(&rx_buf[..len as usize]).unwrap_or("?"),
                        status.rssi,
                        status.snr
                    );
                }
            }
            uart.write(fmt_buf.as_bytes()).await.ok();
            LED_SIGNAL.signal(());
        }
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    loop {
        LED_SIGNAL.wait().await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
    }
}
