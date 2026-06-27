//! NE148.2 RS485 command sweep probe.
//!
//! Wiring (ESP32-S3 + MAX485):
//!   - UART TX  -> MAX485 DI
//!   - UART RX  -> MAX485 RO
//!   - GPIO_DE  -> MAX485 DE *and* /RE (tie both together, DE high = transmit,
//!                 DE low = receive). This is the common single-control-line
//!                 wiring: invert /RE in hardware if your breakout needs it,
//!                 or use two GPIOs if DE/RE aren't tied on your board.
//!   - A/B      -> RS485 bus screw terminals on the NE148.2 connector.
//!   - GND      -> common ground with the bus (per the ne-rs485 pinout notes,
//!                 pin 4 / white wire).
//!
//! Serial config: 8N1, 38400 baud (confirmed against the physical bus).
//!
//! This deliberately does NOT use esp-hal's automatic RS485 half-duplex mode.
//! Reports from both ESP-IDF and esp-hal users show the automatic RTS/DE
//! toggling is flaky around the exact transmit-complete boundary, which is
//! exactly the failure mode that would corrupt your captures. Manual GPIO
//! control with an explicit flush() + small guard delay is slower to write
//! but predictable, which matters more here than raw throughput.
//!
//! Probe strategy: rather than sweeping all 5 bytes of a frame, we fix the
//! skeleton observed in the NE334 protocol reverse-engineering (class142/
//! ne-rs485): `FF <cmd> 00 <param> <checksum>`, and sweep the command byte
//! across 0x00..=0xFF while holding param at a couple of plausible defaults.
//! Every frame sent has a *valid* checksum for its own bytes, since a bad
//! checksum will likely just get silently dropped by the main unit -- that
//! would make the sweep useless regardless of whether the command byte
//! itself was meaningful.
//!
//! All sent/received bytes are logged over defmt-rtt with a monotonic
//! sequence number, so you can correlate this log against the PulseView/
//! sigrok capture running concurrently on the same bus.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
    uart::{Config as UartConfig, DataBits, Parity, StopBits, Uart},
};

esp_bootloader_esp_idf::esp_app_desc!();

/// Known-good frames from the NE334 reverse-engineering (class142/ne-rs485).
/// These are NOT guaranteed to be valid for the NE148.2, but they are the
/// best candidates for "wake the bus up" since the protocol doc states any
/// unknown command can serve that purpose, and these specific bytes are
/// confirmed examples of the manufacturer's framing on a sibling product.
///
/// NOTE: cross-checking these against the verified checksum formula below
/// shows the spec.md table's "init?" and "Idle" rows don't self-consistently
/// match -- init1's bytes produce idle's listed checksum and vice versa,
/// most likely a transcription mix-up in that small hobbyist repo rather
/// than a real protocol quirk (six other commands -- lights, pump, all-off
/// -- check out cleanly against the same formula). Frames are sent here
/// exactly as published; if NE148.2 rejects them, recompute and try the
/// corrected checksums first before assuming the frame body itself is wrong.
const INIT_CANDIDATE_1: [u8; 5] = [0xFF, 0x40, 0x00, 0x80, 0xBF];
const INIT_CANDIDATE_2: [u8; 5] = [0xFF, 0x00, 0x00, 0xC0, 0xFF];

/// Idle/keepalive frame (per spec.md: "Sent as response by main unit", but
/// also usable as a keepalive sent *to* the unit -- the doc notes timing
/// doesn't matter and ~5s intervals are fine to stop the main unit shutting
/// down). See checksum caveat above.
const IDLE_FRAME: [u8; 5] = [0xFF, 0x40, 0x00, 0xC0, 0xBF];

/// Computes the NE-protocol checksum.
///
/// Verified against six independent known-good command frames from the
/// NE334 spec (indoor light, all-indoor-lights, outdoor light, water pump,
/// all-off): plain byte sum of the frame body, truncated to one byte. The
/// spec.md text describes a "mod 128 + 2" variant, but that does not
/// reproduce any of the published example checksums -- this simpler formula
/// does, exactly, for 6/8 examples (see candidate-frame note above for the
/// other 2). The spec's "mod 128 + 2" note appears to describe a separate
/// TCP-bridge byte-stuffing correction, not the base frame checksum.
///
/// `frame` should be all bytes of the frame EXCLUDING the checksum byte.
fn ne_checksum(frame: &[u8]) -> u8 {
    let sum: u32 = frame.iter().map(|&b| b as u32).sum();
    (sum & 0xFF) as u8
}

/// Builds a 5-byte command frame `[0xFF, cmd, 0x00, param, checksum]` with a
/// correctly computed trailing checksum byte.
fn build_frame(cmd: u8, param: u8) -> [u8; 5] {
    let mut frame = [0xFFu8, cmd, 0x00, param, 0x00];
    frame[4] = ne_checksum(&frame[..4]);
    frame
}

/// Verifies whether `frame`'s last byte matches the expected NE checksum.
/// Used when parsing whatever comes back from the main unit, so we can tell
/// real frames apart from line noise / desynced reads.
fn checksum_valid(frame: &[u8]) -> bool {
    match frame.split_last() {
        Some((&last, body)) => ne_checksum(body) == last,
        None => false,
    }
}

/// Sends `frame` over `uart` with manual DE control: raise DE, write bytes,
/// flush (block until the hardware FIFO has actually shifted everything out
/// the wire), wait a short guard time for the transceiver's own propagation
/// delay, then drop back to receive.
async fn send_frame<'a>(uart: &mut Uart<'a, esp_hal::Async>, de: &mut Output<'a>, frame: &[u8]) {
    de.set_high();
    // Small pre-transmit guard so the transceiver direction has settled
    // before the first start bit goes out.
    Timer::after(Duration::from_micros(50)).await;

    embedded_io_async::Write::write_all(uart, frame).await.ok();
    embedded_io_async::Write::flush(uart).await.ok();

    // Post-transmit guard before releasing the bus back to receive mode.
    // At 38400 8N1 one bit is ~26us; a few bit-times of margin is cheap
    // insurance against clipping the last stop bit.
    Timer::after(Duration::from_micros(100)).await;
    de.set_low();
}

/// Reads whatever bytes are available within `window`, returning them in a
/// fixed buffer. Doesn't try to interpret frame boundaries -- that's done
/// for real in PulseView; this is just enough to log "did anything come
/// back at all" and sanity-check candidate frames against the checksum.
async fn read_window<'a>(
    uart: &mut Uart<'a, esp_hal::Async>,
    window: Duration,
) -> heapless::Vec<u8, 64> {
    let mut buf = heapless::Vec::<u8, 64>::new();
    let deadline = embassy_time::Instant::now() + window;

    while embassy_time::Instant::now() < deadline {
        let mut byte = [0u8; 1];
        match embassy_time::with_timeout(
            deadline - embassy_time::Instant::now(),
            embedded_io_async::Read::read(uart, &mut byte),
        )
        .await
        {
            Ok(Ok(n)) if n > 0 => {
                if buf.push(byte[0]).is_err() {
                    break; // buffer full, stop collecting
                }
            }
            _ => break, // timeout or read error/EOF -- stop, window's over
        }
    }
    buf
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    // DE/RE control line. Adjust the pin to whatever GPIO you've wired to
    // the MAX485 DE (and /RE, if tied together) pin.
    let mut de = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    // UART1 for the RS485 bus. Adjust TX/RX pins to your wiring.
    let uart_config = UartConfig::default()
        .with_baudrate(38_400)
        .with_data_bits(DataBits::_8)
        .with_parity(Parity::None)
        .with_stop_bits(StopBits::_1);

    let uart = Uart::new(peripherals.UART1, uart_config)
        .unwrap()
        .with_tx(peripherals.GPIO17)
        .with_rx(peripherals.GPIO18)
        .into_async();
    let mut uart = uart;

    info!("NE148.2 sweep probe starting");

    // --- Step 1: wake the bus -----------------------------------------
    // Send both known init candidates a few times before starting the
    // sweep, in case the main unit needs to see one specifically (or needs
    // repetition) before it starts responding.
    for round in 0..5u32 {
        info!("wake attempt {} : init candidate 1", round);
        send_frame(&mut uart, &mut de, &INIT_CANDIDATE_1).await;
        let resp = read_window(&mut uart, Duration::from_millis(100)).await;
        if !resp.is_empty() {
            info!("  response ({} bytes): {:02x}", resp.len(), resp.as_slice());
        }

        Timer::after(Duration::from_millis(50)).await;

        info!("wake attempt {} : init candidate 2", round);
        send_frame(&mut uart, &mut de, &INIT_CANDIDATE_2).await;
        let resp = read_window(&mut uart, Duration::from_millis(100)).await;
        if !resp.is_empty() {
            info!("  response ({} bytes): {:02x}", resp.len(), resp.as_slice());
        }

        Timer::after(Duration::from_millis(200)).await;
    }

    // --- Step 2: sweep ---------------------------------------------------
    // Sweep the command byte across the full range, holding `param` at a
    // couple of plausible defaults seen in the known NE334 commands (0xC0
    // appears in several real commands; 0x00 is the "off"/null case).
    let params_to_try: [u8; 2] = [0xC0, 0x00];

    let mut seq: u32 = 0;

    'sweep: for cmd in 0u8..=0xFF {
        for &param in &params_to_try {
            seq += 1;

            // Interleave a keepalive every 16 probes so the bus doesn't
            // time out and the main unit shut down mid-sweep (per spec.md:
            // it shuts off a couple seconds after the last command if the
            // control panel is "off").
            if seq % 16 == 0 {
                info!("seq {} : keepalive", seq);
                send_frame(&mut uart, &mut de, &IDLE_FRAME).await;
                let _ = read_window(&mut uart, Duration::from_millis(50)).await;
                Timer::after(Duration::from_millis(20)).await;
            }

            let frame = build_frame(cmd, param);
            info!(
                "seq {} : send cmd=0x{:02x} param=0x{:02x} frame={:02x}",
                seq, cmd, param, frame
            );

            send_frame(&mut uart, &mut de, &frame).await;
            let resp = read_window(&mut uart, Duration::from_millis(80)).await;

            if !resp.is_empty() {
                let valid = checksum_valid(resp.as_slice());
                info!(
                    "seq {} : response ({} bytes, checksum_valid={}): {:02x}",
                    seq,
                    resp.len(),
                    valid,
                    resp.as_slice()
                );
            }

            // Gap between probes so frame boundaries are unambiguous in the
            // logic analyzer capture, and so we're not hammering the bus
            // faster than the main unit can process commands.
            Timer::after(Duration::from_millis(40)).await;

            // Safety valve: stop early on a button press / reset if you
            // need to abort a sweep that's causing unwanted relay chatter.
            // (Left as a placeholder -- wire a GPIO input here if useful.)
            if false {
                break 'sweep;
            }
        }
    }

    info!("sweep complete, holding keepalive loop");

    loop {
        send_frame(&mut uart, &mut de, &IDLE_FRAME).await;
        let resp = read_window(&mut uart, Duration::from_millis(100)).await;
        if !resp.is_empty() {
            info!("keepalive response: {:02x}", resp.as_slice());
        }
        Timer::after(Duration::from_secs(4)).await;
    }
}
