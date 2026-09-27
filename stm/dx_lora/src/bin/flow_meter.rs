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

use core::{
    fmt::Write,
    sync::atomic::{AtomicU32, Ordering},
};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    interrupt,
    mode::Async,
    peripherals::{self},
    spi::{Config as SpiConfig, Spi, mode::Master},
    time::Hertz,
    // usart::{Config as UartConfig, Uart},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use lora_phy::{
    LoRa,
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
    EXTI0 => embassy_stm32::exti::InterruptHandler<interrupt::typelevel::EXTI0>;
});

const FREQUENCY_HZ: u32 = 869_525_000;
// const BAUD_RATE: u32 = 9600;
const OUTPUT_POWER: i32 = 14;

static LED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static EDGE_COUNT: AtomicU32 = AtomicU32::new(0);

type LoraRadio = LoRa<
    Sx126x<
        ExclusiveDevice<Spi<'static, Async, Master>, Output<'static>, Delay>,
        GenericSx126xInterfaceVariant<Output<'static>, ExtiInput<'static, Async>>,
        Sx1262,
    >,
    Delay,
>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // let mut uart_config = UartConfig::default();
    // uart_config.baudrate = BAUD_RATE;
    // let mut uart = Uart::new(
    //     p.USART1,
    //     p.PA10,
    //     p.PA9,
    //     p.DMA1_CH4,
    //     p.DMA1_CH5,
    //     Irqs,
    //     uart_config,
    // )
    // .expect("UART struct");
    //
    // uart.write(b"\r\n").await.ok();
    // uart.write(b"##############################\r\n").await.ok();
    // uart.write(b"LoRa Flow Meter Firmware\r\n").await.ok();
    // uart.write(concat!("Firmware v", env!("CARGO_PKG_VERSION"), "\r\n").as_bytes())
    //     .await
    //     .ok();
    // uart.write(b"##############################\r\n").await.ok();

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
    let spi_dev = ExclusiveDevice::new(spi, nss, Delay).unwrap();

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
    let lora = LoRa::new(Sx126x::new(spi_dev, iv, config), false, Delay)
        .await
        .unwrap();
    spawner.spawn(tx_task(lora)).unwrap();

    // uart.write(b"lora init ok\r\n").await.ok();

    let flow_pin = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up, Irqs);
    spawner.spawn(flow_task(flow_pin)).unwrap();

    let led = Output::new(p.PB11, Level::High, Speed::Low);
    spawner.spawn(led_task(led)).unwrap();

    loop {
        LED_SIGNAL.signal(());
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task]
async fn tx_task(mut lora: LoraRadio) {
    let mut seq: u32 = 0;

    let mdltn_params = lora
        .create_modulation_params(
            SpreadingFactor::_9,
            Bandwidth::_125KHz,
            CodingRate::_4_6,
            FREQUENCY_HZ,
        )
        .unwrap();

    // LoRa TX
    let mut tx_params = lora
        .create_tx_packet_params(8, false, true, false, &mdltn_params)
        .unwrap();

    loop {
        let mut fmt_buf: heapless::String<32> = heapless::String::new();
        let _ = write!(fmt_buf, "{}: {}", seq, EDGE_COUNT.load(Ordering::Relaxed));
        lora.prepare_for_tx(
            &mdltn_params,
            &mut tx_params,
            OUTPUT_POWER,
            fmt_buf.as_bytes(),
        )
        .await
        .unwrap();
        lora.tx().await.unwrap();
        seq += 1;
        LED_SIGNAL.signal(());
        Timer::after(Duration::from_secs(5)).await;
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

// TODO: Look into removing this task and directly using a hardware counter
#[embassy_executor::task]
async fn flow_task(mut flow_pin: ExtiInput<'static, Async>) {
    loop {
        flow_pin.wait_for_falling_edge().await;
        EDGE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
