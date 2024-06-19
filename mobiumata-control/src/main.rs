#![no_std]
#![no_main]

use core::iter::once;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Rgb888;
use mobiumata_common::display::ws2812::Ws2812;
use mobiumata_common::network::{init_network, udp_send, Mode};
use mobiumata_common::state::State;
use mobiumata_control::Buttons;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

const DEBOUNCE_DURATION: u64 = 25;
const PRIMARY_COLOR: Rgb888 = Rgb888::new(255, 255, 255);
const SECONDARY_COLOR: Rgb888 = Rgb888::new(0, 0, 255);

bind_interrupts!(struct Irqs0 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

bind_interrupts!(struct Irqs1 {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let mut pio = Pio::new(p.PIO1, Irqs1);
    let mut ws2812: Ws2812<PIO1, 0, 2> = Ws2812::new(&mut pio.common, pio.sm0, p.DMA_CH1, p.PIN_28);

    ws2812.write(once(SECONDARY_COLOR)).await;

    let mut pio = Pio::new(p.PIO0, Irqs0);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        Output::new(p.PIN_25, Level::High),
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );
    let stack = init_network(
        spawner,
        Mode::Station,
        env!("WIFI_SSID"),
        env!("WIFI_PASSPHRASE"),
        Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 217), 24),
        spi,
        Output::new(p.PIN_23, Level::Low),
    )
    .await;

    ws2812.write(once(PRIMARY_COLOR)).await;

    static SIGNAL: StaticCell<Signal<NoopRawMutex, State>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());

    spawner.spawn(udp_send(stack, signal)).unwrap();

    let mut buttons = Buttons::new(
        p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_0, p.PIN_1,
        p.PIN_10,
    );

    let mut last_broadcast_state = buttons.read_state();
    loop {
        buttons.wait_for_any_edge().await;

        loop {
            let state = buttons.read_state();

            Timer::after_millis(DEBOUNCE_DURATION).await;

            if state == buttons.read_state() {
                if last_broadcast_state != state {
                    last_broadcast_state = state;
                    signal.signal(state);
                }
                break;
            }
        }
    }
}
