//! Wiring
//! ──────
//!   INMP441   ESP32-S3
//!   VDD    →  3.3 V
//!   GND    →  GND
//!   L/R    →  GND       (left channel; 3.3 V for right)
//!   SCK    →  GPIO 14   (BCLK)
//!   WS     →  GPIO 15   (LRCK / word-select)
//!   SD     →  GPIO 4    (data in)

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::dma_buffers;
use esp_hal::i2s::master::{Channels, Config};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    i2s::master::{DataFormat, I2s},
    time::Rate,
};
use rtt_target::rprintln;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", panic_info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const SAMPLE_RATE: u32 = 16_000;
const DMA_BUFFER_SIZE: usize = 4 * 4092;

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

    let (mut rx_buffer, rx_descriptors, _, _) = dma_buffers!(DMA_BUFFER_SIZE, 0);

    let i2s = I2s::new(
        peripherals.I2S0,
        peripherals.DMA_CH0,
        Config::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(SAMPLE_RATE))
            .with_data_format(DataFormat::Data32Channel32)
            .with_channels(Channels::MONO),
    )
    .unwrap();

    let mut i2s_rx = i2s
        .i2s_rx
        .with_bclk(peripherals.GPIO14)
        .with_ws(peripherals.GPIO15)
        .with_din(peripherals.GPIO4)
        .build(rx_descriptors);

    rprintln!("I2S RX ready - {} Hz, 32-bit TDM Philips", SAMPLE_RATE);

    let mut transfer = i2s_rx.read_dma_circular(&mut rx_buffer).unwrap();

    static mut CHUNK: [u8; DMA_BUFFER_SIZE] = [0u8; DMA_BUFFER_SIZE];
    let chunk: &mut [u8] = unsafe { &mut *core::ptr::addr_of_mut!(CHUNK) };

    loop {
        let avail = transfer.available().unwrap();
        if avail == 0 {
            continue;
        }

        transfer.pop(&mut chunk[..avail]).unwrap();

        // INMP441: 24-bit audio, left-aligned in a 32-bit slot → shift >> 16
        let samples = bytemuck_cast(&chunk[..avail]);
        let rms = rms_16(samples);
        rprintln!("samples={} rms={}", samples.len(), rms);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}

fn bytemuck_cast(bytes: &[u8]) -> &[i32] {
    assert_eq!(bytes.len() % 4, 0);
    // SAFETY: DMA buffers are 4-byte aligned; i32 has no invalid bit patterns.
    unsafe { core::slice::from_raw_parts(bytes.as_ptr() as *const i32, bytes.len() / 4) }
}

fn rms_16(samples: &[i32]) -> i64 {
    if samples.is_empty() {
        return 0;
    }
    let sum_sq: i64 = samples
        .iter()
        .map(|&r| {
            let s = (r >> 16) as i16 as i64;
            s * s
        })
        .sum();
    isqrt(sum_sq / samples.len() as i64)
}

fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
