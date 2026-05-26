//! esp_now broadcast example, each connected board runs the same script and has
//! one button and one LED.
//!
//! - on button press the board transmits a 0x01
//! - on button release the board transmits a 0x00
//! - when a board receives a 0x01 transmission, it turns the LED on
//!
//! Wiring
//! ──────
//!
//! GPIO 4  →   Button   →      GND     (INPUT_PULLUP, active LOW)
//! GPIO 5  →   LED      →      330Ω    →   GND

#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker, Timer};
use esp_alloc as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    timer::timg::TimerGroup,
};
use esp_radio::esp_now::{
    BROADCAST_ADDRESS, EspNowManager, EspNowReceiver, EspNowSender, PeerInfo,
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

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

    let button = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::Up),
    );
    let led = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());

    // ESP-NOW Init
    let wifi = peripherals.WIFI;
    let (_controller, interfaces) = esp_radio::wifi::new(wifi, Default::default()).unwrap();
    let esp_now = interfaces.esp_now;
    esp_now.set_channel(11).unwrap();

    info!("esp-now version {}", esp_now.version().unwrap());

    // TODO: Update this to static_cell::make_static
    let (manager, sender, receiver) = esp_now.split();
    let manager = mk_static!(EspNowManager<'static>, manager);
    let sender = mk_static!(
        Mutex::<NoopRawMutex, EspNowSender<'static>>,
        Mutex::<NoopRawMutex, _>::new(sender)
    );

    spawner.spawn(send_button_press(sender, button).expect("To spawn the send task"));
    spawner.spawn(recv_task(manager, receiver, led).expect("To spawn the receive task"));

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn send_button_press(
    sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>,
    button: Input<'static>,
) {
    let mut last_pressed = false;
    let mut send_error = false;
    loop {
        let pressed = button.is_low();
        if pressed != last_pressed || send_error {
            last_pressed = pressed;
            let payload: &[u8] = if pressed { &[1] } else { &[0] };
            let mut sender = sender.lock().await;
            match sender.send_async(&BROADCAST_ADDRESS, payload).await {
                Ok(status) => {
                    send_error = false;
                    info!(
                        "Sent button {} to broadcast={:02x} status={:?}",
                        if pressed { "Down" } else { "Up" },
                        BROADCAST_ADDRESS,
                        status
                    );
                }
                Err(err) => {
                    send_error = true;
                    error!(
                        "broadcast failed dst={:02x} err={:?}",
                        BROADCAST_ADDRESS, err
                    );
                }
            }
        }
        // Debounce the button press
        Timer::after_millis(20).await;
    }
}

#[embassy_executor::task]
async fn recv_task(
    manager: &'static EspNowManager<'static>,
    mut receiver: EspNowReceiver<'static>,
    mut led: Output<'static>,
) {
    loop {
        let msg = receiver.receive_async().await;
        if msg.info.dst_address == BROADCAST_ADDRESS && !manager.peer_exists(&msg.info.src_address)
        {
            manager
                .add_peer(PeerInfo {
                    interface: esp_radio::esp_now::EspNowWifiInterface::Station,
                    peer_address: msg.info.src_address,
                    lmk: None,
                    channel: None,
                    encrypt: false,
                })
                .unwrap();
            info!("Added peer {:02x}", msg.info.src_address);
        }

        if let Some(&state) = msg.data().first() {
            if state == 1 {
                led.set_high();
                info!("Received Button Down Message");
            } else {
                led.set_low();
                info!("Received Button Up Message");
            }
        }
    }
}
