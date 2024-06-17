use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_rp::{
    clocks::RoscRng,
    gpio::Output,
    peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0},
};
use embassy_time::Timer;
use heapless::Vec;
use rand::RngCore;
use static_cell::StaticCell;

pub enum Mode {
    AccessPoint { channel: u8 },
    Station,
}

pub async fn init_network(
    spawner: Spawner,
    mode: Mode,
    ssid: &'static str,
    passphrase: &'static str,
    ip_address: Ipv4Cidr,
    pio_spi: PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    pwr: Output<'static, PIN_23>,
) -> &'static Stack<cyw43::NetDriver<'static>> {
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    let fw: &[u8; 230321] = include_bytes!("../../../../../.cargo/git/checkouts/embassy-9312dcb0ed774b29/a099084/cyw43-firmware/43439A0.bin");
    let clm: &[u8; 4752] = include_bytes!("../../../../../.cargo/git/checkouts/embassy-9312dcb0ed774b29/a099084/cyw43-firmware/43439A0_clm.bin");

    let (net_device, mut control, runner) = cyw43::new(state, pwr, pio_spi, fw).await;
    spawner.must_spawn(wifi_task(runner));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = Config::ipv4_static(StaticConfigV4 {
        address: ip_address,
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

    match mode {
        Mode::AccessPoint { channel } => {
            control
                .start_ap_wpa2(ssid, passphrase, channel)
                .await;
        }
        Mode::Station => {
            control
                .join_wpa2(ssid, passphrase)
                .await
                .expect("join failed");
        }
    }

    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }

    stack
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
