#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod http_handler;
mod wifi;
extern crate alloc;
use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_net::{Config, Stack, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::macros::main;
use esp_hal::peripherals::I2C0;
use esp_wifi::wifi::WifiStaDevice;
use esp_wifi::{initialize, EspWifiInitFor};
use ssd1306::mode::BufferedGraphicsMode;

use esp_hal::i2c::I2C;

use esp_hal::prelude::_embedded_hal_digital_v2_OutputPin;
use esp_hal::system::SystemExt;
use esp_hal::timer::TimerGroup;
use esp_hal::{
    clock::ClockControl, embassy, entry, peripherals::Peripherals, prelude::_fugit_RateExtU32,
};
use esp_hal::{Rng, IO};
use esp_println::println;

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use static_cell::make_static;
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[derive(Clone, Copy)]
struct DisplayController(
    &'static Mutex<
        CriticalSectionRawMutex,
        Ssd1306<
            I2CInterface<I2C<'static, I2C0>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>,
        >,
    >,
);

struct AppState {
    display: DisplayController,
}
impl picoserve::extract::FromRef<AppState> for DisplayController {
    fn from_ref(state: &AppState) -> Self {
        state.display
    }
}

#[main]
async fn main(spawner: Spawner) {
    init_heap();

    println!("Init!");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100.kHz(),
        &clocks,
    );

    let interface = I2CDisplayInterface::new(i2c);

    let mut system_ready_pin = io.pins.gpio2.into_push_pull_output();

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, peripherals.WIFI, WifiStaDevice).unwrap();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let config = Config::dhcpv4(Default::default());

    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.spawn(wifi::connection(controller)).ok();
    spawner.spawn(wifi::net_task(&stack)).ok();

    loop {
        if stack.is_link_up() {
            println!("Link up");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
    })
    .keep_connection_alive());

    let display = DisplayController(make_static!(Mutex::new(
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode()
    )));

    spawner
        .spawn(http_handler::web_task(config, stack, AppState { display }))
        .ok();

    system_ready_pin.set_high().unwrap();

    loop {
        println!("Online");
        Timer::after(Duration::from_secs(5)).await;
    }
}
