#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0, PIO1};
use embassy_rp::pio::{Instance, InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Rgb888;
use heapless::Vec;
use mobiumata_common::state::State;
use mobiumata_control::Buttons;
use mobiumata_ws2812::Ws2812;
use rand::RngCore;
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
async fn udp_send(
    stack: &'static Stack<cyw43::NetDriver<'static>>,
    signal: &'static Signal<NoopRawMutex, State>,
) {
    let mut rx_buffer = [0; 1024];
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_buffer = [0; 1024];
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
        let state = signal.wait().await;
        let mut buffer = [0; 1024];

        let size = serde_json_core::to_slice(&state, &mut buffer).expect("serialize failed");
        socket
            .send_to(&buffer[..size], (Ipv4Address::BROADCAST, 1234))
            .await
            .expect("send failed");

        info!("State: {:?}", state);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO1, Irqs1);

    let mut ws2812: Ws2812<PIO1, 0, 2> = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_28);

    led(&mut ws2812, hsv(128 + 0, 255, 255)).await;

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
    led(&mut ws2812, hsv(128 + 10, 255, 255)).await;

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    led(&mut ws2812, hsv(128 + 20, 255, 255)).await;

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 217), 24),
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

    led(&mut ws2812,hsv(128 + 30, 255, 255)).await;

    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }

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
