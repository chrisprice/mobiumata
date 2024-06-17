#![no_std]
#![no_main]

use core::array;

use core::fmt::Write;
use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0, PIO1};
use embassy_rp::pio::{Instance, InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{self, Channel, Sender};
use embassy_sync::signal::{self, Signal};
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::mono_font::ascii::{FONT_5X8, FONT_6X10};
use embedded_graphics::mono_font::iso_8859_14::FONT_4X6;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use heapless::{String, Vec};
use mobiumata_common::automaton::{ElementaryCellularAutomaton, Rule, Wrap};
use mobiumata_common::display::{Display, HEIGHT, NUM_LEDS, WIDTH};
use mobiumata_common::state::State;
use mobiumata_ws2812::Ws2812;
use rand::{Rng, RngCore};
use smart_leds::hsv::{hsv2rgb, Hsv};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs0 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

bind_interrupts!(struct Irqs1 {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

const NUM_LEDS_PER_PIN: usize = NUM_LEDS / 2;
const BRIGHTNESS: u8 = 16;

fn hsv(hue: u8, sat: u8, val: u8) -> Rgb888 {
    let rgb = hsv2rgb(Hsv {
        hue,
        sat,
        val: (val as u16 * (BRIGHTNESS as u16 + 1) / 256) as u8,
    });
    Rgb888::new(rgb.r, rgb.g, rgb.b)
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn udp_listen(
    stack: &'static Stack<cyw43::NetDriver<'static>>,
    signal: &'static Signal<NoopRawMutex, State>,
) {
    let mut rx_buffer = [0; 4096];
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(1234).expect("bind failed");

    loop {
        let mut buffer = [0; 1024];
        let (len, addr) = socket.recv_from(&mut buffer).await.expect("recv failed");
        info!("received {:?} from {:?}", buffer[..len], addr);
        let (state, _) = serde_json_core::from_slice(&buffer[..len]).expect("deserialize failed");
        signal.signal(state);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common,
        sm0,
        sm1,
        ..
    } = Pio::new(p.PIO1, Irqs1);

    let mut ws2812_1: Ws2812<PIO1, 0, NUM_LEDS_PER_PIN> =
        Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_27);
    let mut ws2812_2: Ws2812<PIO1, 1, NUM_LEDS_PER_PIN> =
        Ws2812::new(&mut common, sm1, p.DMA_CH2, p.PIN_26);

    led(&mut ws2812_1, &mut ws2812_2, hsv(128 + 0, 255, 255)).await;

    let fw: &[u8; 230321] = include_bytes!("../../../../.cargo/git/checkouts/embassy-9312dcb0ed774b29/a099084/cyw43-firmware/43439A0.bin");
    let clm: &[u8; 4752] = include_bytes!("../../../../.cargo/git/checkouts/embassy-9312dcb0ed774b29/a099084/cyw43-firmware/43439A0_clm.bin");

    let mut pio = Pio::new(p.PIO0, Irqs0);

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.must_spawn(wifi_task(runner));
    led(&mut ws2812_1, &mut ws2812_2, hsv(128 + 10, 255, 255)).await;

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    led(&mut ws2812_1, &mut ws2812_2, hsv(128 + 20, 255, 255)).await;

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 218), 24),
        gateway: None,
        dns_servers: Vec::new(),
    });

    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        RoscRng.next_u64(),
    ));

    spawner.must_spawn(net_task(stack));

    control
        .join_wpa2(env!("WIFI_SSID"), env!("WIFI_PASSPHRASE"))
        .await
        .expect("join failed");

    led(&mut ws2812_1, &mut ws2812_2, hsv(128 + 30, 255, 255)).await;

    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }

    led(&mut ws2812_1, &mut ws2812_2, hsv(128 + 40, 255, 255)).await;

    static SIGNAL: StaticCell<Signal<NoopRawMutex, State>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());

    spawner.spawn(udp_listen(stack, signal)).unwrap();

    let mut display = Display::new();
    static UNIVERSE: StaticCell<[[bool; WIDTH]; HEIGHT]> = StaticCell::new();
    let universe = UNIVERSE.init(array::from_fn(|_| array::from_fn(|_| RoscRng.gen_bool(0.5))));
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
                        // delta if delta > -16 && delta < 0 => hsv(hue, (-delta * 16) as u8, 255),
                        // 0 => hsv(hue, 0, 255),
                        delta if delta > 0 && delta < 16 => hsv(hue, 255, (delta * 16) as u8),
                        _ => hsv(hue, 255, 255),
                    };
                    Pixel(Point::new(y as i32, (WIDTH - 1 - x) as i32), color)
                })
            });
            display.draw_iter(pixels).unwrap();

            join(
                ws2812_1.write(display.data(0..NUM_LEDS_PER_PIN)),
                ws2812_2.write(display.data(NUM_LEDS_PER_PIN..NUM_LEDS)),
            )
            .await;

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

async fn led(
    ws2812_1: &mut Ws2812<'_, impl Instance, 0, NUM_LEDS_PER_PIN>,
    ws2812_2: &mut Ws2812<'_, impl Instance, 1, NUM_LEDS_PER_PIN>,
    color: Rgb888,
) {
    let data = [color; NUM_LEDS_PER_PIN];
    ws2812_1.write(data.iter().copied()).await;
    ws2812_2.write(data.iter().copied()).await;
}
