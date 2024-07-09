#![no_std]
#![no_main]

use core::array;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::{Ipv4Address, Ipv4Cidr};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use mobiumata_common::automaton::ElementaryCellularAutomaton;
use mobiumata_common::display::ws2812::Ws2812;
use mobiumata_common::display::{Display, HEIGHT, WIDTH};
use mobiumata_common::network::{init_network, udp_listen, Mode};
use mobiumata_common::state::{State, Step};
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
        Mode::AccessPoint { channel: 5 },
        env!("WIFI_SSID"),
        env!("WIFI_PASSPHRASE"),
        Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 218), 24),
        spi,
        Output::new(p.PIN_23, Level::Low),
    )
    .await;

    static SIGNAL: StaticCell<Signal<NoopRawMutex, State>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());

    spawner.spawn(udp_listen(stack, signal)).unwrap();

    static UNIVERSE: StaticCell<[[bool; WIDTH]; HEIGHT]> = StaticCell::new();
    let universe = UNIVERSE.init(array::from_fn(|_| {
        array::from_fn(|_| RoscRng.gen_bool(0.5))
    }));
    let mut state = State::default();
    let mut ticker = RunStepTicker::new(state.step);

    spawner.spawn(idle_reset(signal)).unwrap();

    loop {
        for y_update in 0..HEIGHT {
            if let Some(new_state) = signal.try_take() {
                state = new_state;
                info!("New state: {:?}", state);
            }

            let automaton = ElementaryCellularAutomaton::new(state.wrap, state.rule);
            automaton.next_row(universe, y_update);

            let pixels = universe.iter().enumerate().flat_map(|(y, row)| {
                row.iter().enumerate().map(move |(x, cell)| {
                    let mut hue = 15;
                    let saturation = 255;
                    let value = 255;
                    if *cell {
                        hue = 170;
                    }
                    Pixel(
                        Point::new(y as i32, (WIDTH - 1 - x) as i32),
                        hsv(hue, saturation, value),
                    )
                })
            });

            display.draw_iter(pixels).unwrap();
            display.flush().await;

            ticker.next(state.step).await;
        }
    }
}

struct RunStepTicker {
    ticker: Ticker,
    step: Step,
}

impl RunStepTicker {
    const RUN_TICK_DURATION: Duration = Duration::from_millis(10);
    const STEP_TICK_DURATION: Duration = Duration::from_secs(1);

    fn new(step: Step) -> Self {
        Self {
            ticker: Ticker::every(if step.inner() {
                Self::STEP_TICK_DURATION
            } else {
                Self::RUN_TICK_DURATION
            }),
            step,
        }
    }
    async fn next(&mut self, step: Step) {
        if self.step != step {
            *self = Self::new(step);
        }
        self.ticker.next().await
    }
}

#[embassy_executor::task]
async fn idle_reset(signal: &'static Signal<NoopRawMutex, State>) {
    const IDLE_RESET_DURATION: Duration = Duration::from_secs(30);
    loop {
        if let Either::First(_) = select(Timer::after(IDLE_RESET_DURATION), signal.wait()).await {
            info!("Idle");
            signal.signal(State::default());
        } else {
            info!("Reset timer");
        }
    }
}
