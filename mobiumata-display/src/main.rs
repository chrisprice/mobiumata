#![no_std]
#![no_main]

use core::array;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use mobiumata_common::automaton::ElementaryCellularAutomaton;
use mobiumata_common::display::ws2812::Ws2812;
use mobiumata_common::display::{Display, HEIGHT, WIDTH};
use mobiumata_common::network::{init_network, udp_listen, Mode};
use mobiumata_common::state::State;
use rand::Rng;
use smart_leds::hsv::{hsv2rgb, Hsv};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs0 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

bind_interrupts!(struct Irqs1 {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

const PRIMARY_COLOR: Rgb888 = Rgb888::new(BRIGHTNESS, BRIGHTNESS, BRIGHTNESS);
const SECONDARY_COLOR: Rgb888 = Rgb888::new(0, 0, BRIGHTNESS);

const BRIGHTNESS: u8 = 16;

fn hsv(hue: u8, sat: u8, val: u8) -> Rgb888 {
    let rgb = hsv2rgb(Hsv {
        hue,
        sat,
        val: (val as u16 * (BRIGHTNESS as u16 + 1) / 256) as u8,
    });
    Rgb888::new(rgb.r, rgb.g, rgb.b)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let mut pio = Pio::new(p.PIO1, Irqs1);
    let mut display = Display::new(
        Ws2812::new(&mut pio.common, pio.sm0, p.DMA_CH1, p.PIN_27),
        Ws2812::new(&mut pio.common, pio.sm1, p.DMA_CH2, p.PIN_26),
    );

    display.clear(SECONDARY_COLOR).unwrap();

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
        Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 218), 24),
        spi,
        Output::new(p.PIN_23, Level::Low),
    )
    .await;

    display.clear(PRIMARY_COLOR).unwrap();

    static SIGNAL: StaticCell<Signal<NoopRawMutex, State>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());

    spawner.spawn(udp_listen(stack, signal)).unwrap();

    static UNIVERSE: StaticCell<[[bool; WIDTH]; HEIGHT]> = StaticCell::new();
    let universe = UNIVERSE.init(array::from_fn(|_| {
        array::from_fn(|_| RoscRng.gen_bool(0.5))
    }));
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut state = State::default();

    loop {
        for y_update in 0..HEIGHT {
            if let Some(new_state) = signal.try_take() {
                state = new_state;
                info!("New state: {:?}", state);
            }

            let automaton = ElementaryCellularAutomaton::new(state.wrap, state.rule);
            automaton.next_row(universe, y_update);

            display.clear(Rgb888::new(0, 0, 0)).unwrap();

            let pixels = universe.iter().enumerate().flat_map(|(y, row)| {
                row.iter().enumerate().map(move |(x, cell)| {
                    let hue = if *cell { 170 } else { 20 };
                    let color = match (y as isize - y_update as isize) % HEIGHT as isize {
                        delta if delta > 0 && delta < 16 => hsv(hue, 255, (delta * 16) as u8),
                        _ => hsv(hue, 255, 255),
                    };
                    Pixel(Point::new(y as i32, (WIDTH - 1 - x) as i32), color)
                })
            });
            display.draw_iter(pixels).unwrap();

            display.flush().await;

            if state.step.inner() {
                for _ in 0..1000 {
                    ticker.next().await;
                    if signal.signaled() {
                        break;
                    }
                }
            } else {
                ticker.next().await;
            }
        }
    }
}
