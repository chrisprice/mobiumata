#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::{Instance, InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Rgb888;
use mobiumata_common::network::{init_network, udp_send, Mode};
use mobiumata_common::state::State;
use mobiumata_control::Buttons;
use mobiumata_ws2812::Ws2812;
use smart_leds::hsv::{hsv2rgb, Hsv};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

const DEBOUNCE_DURATION: u64 = 25;
const BRIGHTNESS: u8 = 255;

fn hsv(hue: u8, sat: u8, val: u8) -> Rgb888 {
    let rgb = hsv2rgb(Hsv {
        hue,
        sat,
        val: (val as u16 * (BRIGHTNESS as u16 + 1) / 256) as u8,
    });
    Rgb888::new(rgb.r, rgb.g, rgb.b)
}

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

    led(&mut ws2812, hsv(128 + 0, 255, 255)).await;

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

    led(&mut ws2812, hsv(128 + 40, 255, 255)).await;

    static SIGNAL: StaticCell<Signal<NoopRawMutex, State>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());

    spawner.spawn(udp_send(stack, signal)).unwrap();

    let mut buttons = Buttons::new(
        p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_0, p.PIN_1,
        p.PIN_10,
    );

    let mut state_1 = buttons.read_state();
    loop {
        buttons.wait_for_any_edge().await;

        Timer::after_millis(DEBOUNCE_DURATION).await;

        let state_2 = buttons.read_state();

        if state_1 != state_2 {
            signal.signal(state_2);
            state_1 = state_2;
        }
    }
}

async fn led(ws2812: &mut Ws2812<'_, impl Instance, 0, 2>, color: Rgb888) {
    let data = [
        color,
        Rgb888::new(255, 0, 0), // WHITE BYTE
    ];
    ws2812.write(data.into_iter()).await;
}
