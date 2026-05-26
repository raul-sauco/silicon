//! esp_now broadcast example, each connected board sends and listen to
//! messages forwarding them to serial

#![no_std]
#![no_main]

use defmt::{error, info, trace};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use esp_alloc as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
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

    let wifi = peripherals.WIFI;
    // start the controller in station mode
    let (_controller, interfaces) = esp_radio::wifi::new(wifi, Default::default()).unwrap();

    let esp_now = interfaces.esp_now;
    esp_now.set_channel(11).unwrap();

    info!("esp-now version {}", esp_now.version().unwrap());

    let (manager, sender, receiver) = esp_now.split();
    let manager = mk_static!(EspNowManager<'static>, manager);
    let sender = mk_static!(
        Mutex::<NoopRawMutex, EspNowSender<'static>>,
        Mutex::<NoopRawMutex, _>::new(sender)
    );

    spawner.spawn(listener(manager, receiver).unwrap());
    spawner.spawn(broadcaster(sender).unwrap());

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        ticker.next().await;
        let peer = match manager.fetch_peer(false) {
            Ok(peer) => peer,
            Err(_) => {
                if let Ok(peer) = manager.fetch_peer(true) {
                    peer
                } else {
                    continue;
                }
            }
        };

        trace!("Send hello to peer {:02x}", peer.peer_address);
        let mut sender = sender.lock().await;
        match sender.send_async(&peer.peer_address, b"Hello Peer.").await {
            Ok(_status) => info!("Send Hello To Peer: Done"),
            Err(_err) => error!("Send Hello To Peer: Error"),
        }
    }
}

#[embassy_executor::task]
async fn broadcaster(sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>) {
    let mut ticker = Ticker::every(Duration::from_secs(5));
    loop {
        ticker.next().await;

        info!("Send Broadcast...");
        let mut sender = sender.lock().await;
        match sender.send_async(&BROADCAST_ADDRESS, b"Hello.").await {
            Ok(status) => {
                info!(
                    "broadcast ok dst={:02x} status={:?}",
                    BROADCAST_ADDRESS, status
                );
            }
            Err(err) => {
                error!(
                    "broadcast failed dst={:02x} err={:?}",
                    BROADCAST_ADDRESS, err
                );
            }
        }
    }
}

#[embassy_executor::task]
async fn listener(manager: &'static EspNowManager<'static>, mut receiver: EspNowReceiver<'static>) {
    loop {
        let r = receiver.receive_async().await;
        info!("Received {:a}", r.data());
        if r.info.dst_address == BROADCAST_ADDRESS && !manager.peer_exists(&r.info.src_address) {
            manager
                .add_peer(PeerInfo {
                    interface: esp_radio::esp_now::EspNowWifiInterface::Station,
                    peer_address: r.info.src_address,
                    lmk: None,
                    channel: None,
                    encrypt: false,
                })
                .unwrap();
            info!("Added peer {:02x}", r.info.src_address);
        }
    }
}
